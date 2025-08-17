import os
import secrets
import dateutil
from datetime import datetime
from .file import DbFile
from .user import Config, User
from .expenses import ExpenseRecord, YearExpensesReport
from .savings import Savings
from .categories import CategoryType, YearCategories


class TinyExpensesConfig(Config):
    def __init__(self, directory):
        super().__init__(directory)

        if self._data.get("tinyexpenses", None) is None:
            self._data["tinyexpenses"] = {
                "currency": "",
                "api_token": "",
            }
            self._save()

        self._app_config = self._data["tinyexpenses"]

    def get_currency(self) -> str:
        return self._app_config.get("currency", "")

    def set_currency(self, currency: str) -> None:
        self._app_config["currency"] = currency
        self._save()

    def set_token(self):
        self._app_config["api_token"] = secrets.token_urlsafe(32)
        self._save()
        return self.get_token()

    def get_token(self):
        return self._data["tinyexpenses"].get("api_token", None)


class AppUser(User):
    EXPENSES_FILE_NAME = "expenses.csv"
    CATEGORIES_FILE_NAME = "categories.csv"
    SAVINGS_FILE_NAME = "savings.csv"
    APP_DIRECTORY = "tinyexpenses"

    def __init__(self, id, user_directory):
        super().__init__(id, TinyExpensesConfig(user_directory))

        self._app_path = os.path.join(user_directory, self.APP_DIRECTORY)

    @property
    def currency(self):
        return self._config.get_currency()

    def set_currency(self, currency: str):
        self._config.set_currency(currency)

    def set_token(self):
        return self._config.set_token()

    def get_token(self):
        return self._config.get_token()

    def get_available_expenses_files(self) -> list[int]:
        if not os.path.exists(self._app_path):
            return []

        available_years = []

        for entry in os.scandir(self._app_path):
            if entry.is_dir() and entry.name.isdigit():
                if self._get_year_expenses_file(entry.name).exists():
                    available_years.append(int(entry.name))

        return sorted(available_years)

    def get_available_categories_files(self) -> list[int]:
        if not os.path.exists(self._app_path):
            return []

        available_years = []

        for entry in os.scandir(self._app_path):
            if entry.is_dir() and entry.name.isdigit():
                if self._get_year_categories_file(entry.name).exists():
                    available_years.append(int(entry.name))

        return sorted(available_years)

    def _get_year_expenses_file(self, year: str | int) -> DbFile:
        return DbFile(
            os.path.join(
                self._app_path,
                str(year),
                self.EXPENSES_FILE_NAME,
            )
        )

    def get_year_expenses(self, year: str | int) -> YearExpensesReport:
        return YearExpensesReport(self._get_year_expenses_file(year))

    def _get_year_categories_file(self, year: str | int) -> DbFile:
        return DbFile(
            os.path.join(
                self._app_path,
                str(year),
                self.CATEGORIES_FILE_NAME,
            )
        )

    def get_year_categories(self, year: str | int) -> YearCategories:
        return YearCategories(self._get_year_categories_file(year))

    def get_savings(self) -> Savings:
        db_file = DbFile(os.path.join(self._app_path, self.SAVINGS_FILE_NAME))

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
                user_directory = os.path.join(db_path, entry.name)
                if not Config.config_file_exists(user_directory):
                    continue

                user = AppUser(id=entry.name, user_directory=user_directory)

                self._users_db[user.id] = user

    def get(self, username) -> AppUser | None:
        return self._users_db.get(username, None)
