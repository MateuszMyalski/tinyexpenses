import os
import flask_login
import secrets
import dateutil
import tomllib
import tomli_w
from datetime import datetime
from .file import DbFile
from .expenses import ExpenseRecord, YearExpensesReport
from .savings import Savings
from .categories import CategoryType, YearCategories
from werkzeug.security import check_password_hash, generate_password_hash


class Config:
    CONFIG_FILE_NAME = "config.toml"

    def __init__(self, db_file: DbFile) -> None:
        self._db_file = db_file
        self._data = self._load()

    def _load(self) -> dict:
        if not self._db_file.exists():
            raise FileNotFoundError(f"Missing config: {self._db_file.get_path()}")
        with open(self._db_file.get_path(), "rb") as f:
            return tomllib.load(f)

    def _save(self) -> None:
        with open(self._db_file.get_path(), "wb") as f:
            tomli_w.dump(self._data, f)

    def get_username(self) -> str:
        return self._data["user"]["username"]

    def get_full_name(self) -> str:
        return self._data["user"].get("full_name", "")

    def set_full_name(self, full_name: str) -> None:
        self._data["user"]["full_name"] = full_name
        self._save()

    def get_currency(self) -> str:
        return self._data["user"].get("currency", "")

    def set_currency(self, currency: str) -> None:
        self._data["user"]["currency"] = currency
        self._save()

    def get_password_hash(self) -> str:
        return self._data["user"]["password_hash"]

    def check_password(self, password: str) -> bool:
        return check_password_hash(self.get_password_hash(), password)

    def change_password(self, current_password: str, new_password: str) -> bool:
        if not self.check_password(current_password):
            return False

        self._data["user"]["password_hash"] = generate_password_hash(new_password)
        self._save()

        return True

    def set_token(self):
        self._data["user"]["api_token"] = secrets.token_urlsafe(32)
        self._save()

        return self.get_token()

    def get_token(self):
        return self._data["user"].get("api_token", None)


class ConfigCreator(Config):
    def __init__(self, db_file: DbFile):
        if not os.path.exists(db_file._dir):
            raise FileNotFoundError(f"Provided dir does not exist: {db_file._dir}")

        self._data = {
            "user": {
                "username": "",
                "full_name": "",
                "active": True,
                "password_hash": "",
                "currency": "",
                "api_token": "",
            }
        }

        # Do not overwrite config that already exists
        if not db_file.exists():
            with open(db_file.get_path(), "wb") as f:
                tomli_w.dump(self._data, f)

        super().__init__(db_file)

    def set_username(self, username: str):
        self._data["user"]["username"] = username
        self._save()

    def set_password(self, password: str):
        self._data["user"]["password_hash"] = generate_password_hash(password)
        self._save()

    def generate_api_token(self) -> str:
        token = secrets.token_urlsafe(32)
        self._data["user"]["api_token"] = token
        self._save()
        return token


class User(flask_login.UserMixin):
    EXPENSES_FILE_NAME = "expenses.csv"
    CATEGORIES_FILE_NAME = "categories.csv"
    SAVINGS_FILE_NAME = "savings.csv"

    def __init__(self, id, config: Config):
        self.id = id
        self.config = config
        self.user_directory = ""  # Set later in Users.load()

    @property
    def username(self):
        return self.config.get_username()

    @property
    def full_name(self):
        return self.config.get_full_name()

    @property
    def currency(self):
        return self.config.get_currency()

    def check_password(self, password: str) -> bool:
        return self.config.check_password(password)

    def set_password(self, current: str, new: str) -> bool:
        return self.config.change_password(current, new)

    def set_full_name(self, name: str):
        self.config.set_full_name(name)

    def set_currency(self, currency: str):
        self.config.set_currency(currency)

    def get_id(self):
        return self.id

    def set_token(self):
        return self.config.set_token()

    def get_token(self, token):
        return self.config.get_token()

    def get_available_expenses_files(self) -> list[int]:
        if not os.path.exists(self.user_directory):
            return []

        available_years = []

        for entry in os.scandir(self.user_directory):
            if entry.is_dir() and entry.name.isdigit():
                if self._get_year_expenses_file(entry.name).exists():
                    available_years.append(int(entry.name))

        return sorted(available_years)

    def get_available_categories_files(self) -> list[int]:
        if not os.path.exists(self.user_directory):
            return []

        available_years = []

        for entry in os.scandir(self.user_directory):
            if entry.is_dir() and entry.name.isdigit():
                if self._get_year_categories_file(entry.name).exists():
                    available_years.append(int(entry.name))

        return sorted(available_years)

    def _get_year_expenses_file(self, year: str | int) -> DbFile:
        return DbFile(
            os.path.join(self.user_directory, str(year), self.EXPENSES_FILE_NAME)
        )

    def get_year_expenses(self, year: str | int) -> YearExpensesReport:
        return YearExpensesReport(self._get_year_expenses_file(year))

    def _get_year_categories_file(self, year: str | int) -> DbFile:
        return DbFile(
            os.path.join(self.user_directory, str(year), self.CATEGORIES_FILE_NAME)
        )

    def get_year_categories(self, year: str | int) -> YearCategories:
        return YearCategories(self._get_year_categories_file(year))

    def get_savings(self) -> Savings:
        db_file = DbFile(os.path.join(self.user_directory, self.SAVINGS_FILE_NAME))

        return Savings(db_file)

    def create_year_categories_file(
        self, year: str | int, template_year: str | int | None = None
    ) -> None:
        try:
            escaped_year = int(year)
            dateutil.parser.parse(f"{escaped_year}-01-01")
        except Exception as e:
            raise NameError(f"Year number looks odd : {e}")

        categories_file = self._get_year_categories_file(escaped_year)

        if template_year is None:
            categories_file.create()
        else:
            template_file = self._get_year_categories_file(template_year)
            categories_file.copy_from(template_file.get_path())

    def create_year_expenses(self, year: str | int, initial_balance: float) -> None:
        try:
            escaped_year = int(year)

            initial_balance_entry = ExpenseRecord(
                timestamp=datetime.now(),
                category=CategoryType.INITIAL_BALANCE_LABEL.value,
                expense_date=f"{escaped_year}-01-01",
                amount=initial_balance,
                description=CategoryType.INITIAL_BALANCE_LABEL.value,
            )
        except Exception as e:
            raise NameError(f"Year number looks odd : {e}")

        expenses_file = self._get_year_expenses_file(escaped_year)
        expenses_file.create()

        year_expenses = self.get_year_expenses(escaped_year)
        year_expenses.insert_expense(initial_balance_entry)


class Users:
    def __init__(self):
        self._users_db = {}

    def load(self, db_path: str) -> None:
        if not os.path.exists(db_path):
            raise FileNotFoundError(f"Database path {db_path} does not exist.")

        with os.scandir(db_path) as entries:
            for entry in entries:
                user_cfg_file = DbFile(
                    os.path.join(db_path, entry.name, Config.CONFIG_FILE_NAME)
                )

                user = self._load_user(user_cfg_file)
                if user is None:
                    continue

                self._users_db[user.id] = user
                user.user_directory = os.path.join(db_path, entry.name)

    def _load_user(self, db_file: DbFile) -> User | None:
        if not db_file.exists():
            return None

        config = Config(db_file)
        user = User(id=config.get_username(), config=config)

        return user

    def get(self, username) -> User | None:
        return self._users_db.get(username, None)
