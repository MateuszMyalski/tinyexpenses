import werkzeug.security
import csv
import dateutil.parser
import flask_login
import tomllib
import os
import calendar
from dataclasses import dataclass, field
from enum import Enum


class ExpenseRecord:
    def __init__(
        self, timestamp: str, category: str, date: str, amount: str, description: str
    ):
        self.timestamp = dateutil.parser.parse(timestamp)
        self.date = dateutil.parser.parse(date).date()
        self.description = description
        self.amount = float(amount.replace(",", "."))
        self.category = category

    def __str__(self) -> str:
        return f"{self.timestamp} | {self.category} | {self.date} | {self.amount} | {self.description}"


class User(flask_login.UserMixin):
    full_name = None
    directory = str()
    currency = str()

    def __init__(self, id, username, password_hash):
        self.id = id
        self.username = username
        self.password_hash = password_hash

    def check_password(self, password):
        return werkzeug.security.check_password_hash(self.password_hash, password)

    def get_id(self):
        return self.id

    def get_available_expenses_year_reports(self) -> list[int]:
        if not os.path.exists(self.directory):
            return []

        available_years = []
        for entry in os.scandir(self.directory):
            if entry.is_dir() and entry.name.isdigit():
                try:
                    self.get_expenses_report_file(int(entry.name))
                except FileNotFoundError:
                    continue
                available_years.append(int(entry.name))

        return sorted(available_years)

    def get_expenses_report_file(self, year: int) -> str:
        expense_report_path = os.path.join(self.directory, str(year), "expenses.csv")
        if not os.path.exists(expense_report_path):
            raise FileNotFoundError(
                f"Expense report file for year {year} does not exist at {expense_report_path}"
            )
        return expense_report_path

    def get_registered_categories_file(self) -> str:
        categories_file_path = os.path.join(self.directory, "categories.csv")
        if not os.path.exists(categories_file_path):
            raise FileNotFoundError(
                f"Categories file does not exist at {categories_file_path}"
            )
        return categories_file_path


class CategoryType(Enum):
    SAVINGS = "Savings"
    WANTS = "Wants"
    NEEDS = "Needs"
    INCOME = "Income"


class Categories:
    def __init__(self, account: User):
        self.account = account
        self.categories = []
        self.category_types = {}

        self._load_categories()

    def _load_categories(self) -> None:
        try:
            self.filename = self.account.get_registered_categories_file()
        except FileNotFoundError as e:
            raise e

        with open(self.filename, mode="r", newline="") as file:
            reader = csv.reader(file)
            for idx, row in enumerate(reader):
                if len(row) != 2:
                    continue

                category = row[0].strip()
                cat_type = row[1].strip().title()

                if not any(cat_type == item.value for item in CategoryType):
                    print(f"Invalid category type {file.name}:{idx} - '{cat_type}'")
                    continue

                self.category_types.setdefault(cat_type, []).append(category)

    def get_categories_savings(self) -> list[str]:
        return self.category_types.get(CategoryType.SAVINGS.value, [])

    def get_categories_wants(self) -> list[str]:
        return self.category_types.get(CategoryType.WANTS.value, [])

    def get_categories_needs(self) -> list[str]:
        return self.category_types.get(CategoryType.NEEDS.value, [])

    def get_categories_income(self) -> list[str]:
        return self.category_types.get(CategoryType.INCOME.value, [])

    def is_expense_category(self, category: str) -> bool:
        return any(
            category in categories
            for cat_type, categories in self.category_types.items()
            if cat_type != CategoryType.INCOME.value
        )

    def is_income_category(self, category: str) -> bool:
        if category in self.category_types.get(CategoryType.INCOME.value, []):
            return True
        return False

    def get_categories(self) -> list[str]:
        return sum(self.category_types.values(), [])

    def get_categories_by_type(self) -> dict[str, list[str]]:
        return self.category_types


@dataclass
class YearExpensesTotals:
    totals: list[float] = field(
        default_factory=lambda: [0.0] * (len(calendar.month_name) - 1)
    )

    def __post_init__(self):
        if len(self.totals) != (len(calendar.month_name) - 1):
            raise ValueError(
                f"Totals must have exactly {len(calendar.month_name) - 1} elements"
            )

    def __getitem__(self, month: int) -> float:
        return self.totals[month]

    def __setitem__(self, month: int, value: float) -> None:
        self.totals[month] = value

    def __add__(self, other: "YearExpensesTotals") -> "YearExpensesTotals":
        if not isinstance(other, YearExpensesTotals):
            return NotImplemented
        if len(self.totals) != len(other.totals):
            raise ValueError("Cannot add YearExpensesTotals of different lengths.")

        return YearExpensesTotals([a + b for a, b in zip(self.totals, other.totals)])
    
    def __iadd__(self, other: "YearExpensesTotals") -> "YearExpensesTotals":
        if not isinstance(other, YearExpensesTotals):
            return NotImplemented
        if len(self.totals) != len(other.totals):
            raise ValueError("Cannot add YearExpensesTotals of different lengths.")

        for i in range(len(self.totals)):
            self.totals[i] += other.totals[i]
        return self

    def __sub__(self, other: "YearExpensesTotals") -> "YearExpensesTotals":
        if not isinstance(other, YearExpensesTotals):
            return NotImplemented
        if len(self.totals) != len(other.totals):
            raise ValueError("Cannot subtract YearExpensesTotals of different lengths.")

        return YearExpensesTotals([a - b for a, b in zip(self.totals, other.totals)])
    
    def __mul__(self, other: float | int) -> "YearExpensesTotals":
        if not isinstance(other, (float, int)):
            return NotImplemented
        
        return YearExpensesTotals([a * other for a in self.totals])

    def __iter__(self):
        # Only iterate over months 1-12 (skip index 0, which is empty string in calendar.month_name)
        return iter(self.totals)


class YearExpensesReport:
    INITIAL_BALANCE_LABEL = "Initial Balance"

    def __init__(self, account: User, year: int):
        self.account: User = account
        self.expenses_records: list[ExpenseRecord] = list()
        self.expenses_by_category: dict[str, list[ExpenseRecord]] = dict()
        self.expenses_by_category_monthly_totals: dict[str, YearExpensesTotals] = dict()
        self.initial_balance: float = 0.0

        self._load_expenses(year)
        self._sum_expenses_by_category_per_month()

    def _load_expenses(self, year: int) -> None:
        try:
            self.filename = self.account.get_expenses_report_file(year)
        except FileNotFoundError as e:
            raise e

        with open(self.filename, mode="r", newline="") as file:
            reader = csv.reader(file)

            for row, record in enumerate(reader):
                if len(record) != 5:
                    continue

                try:
                    expense = ExpenseRecord(
                        timestamp=record[0],
                        category=record[1],
                        date=record[2],
                        amount=record[3],
                        description=record[4],
                    )
                except (IndexError, ValueError) as reason:
                    print(f"Cannot parse: {file.name}:{row} - {reason}.")
                    continue

                if expense.category == self.INITIAL_BALANCE_LABEL:
                    self.initial_balance = expense.amount
                    continue

                self.expenses_records.append(expense)
                self.expenses_by_category.setdefault(expense.category, []).append(
                    expense
                )

    def _sum_expenses_by_category_per_month(self) -> None:
        for category, expenses in self.expenses_by_category.items():
            for expense in expenses:
                self.expenses_by_category_monthly_totals.setdefault(
                    category, YearExpensesTotals()
                ).totals[expense.date.month - 1] += expense.amount

    def get_expenses_by_category_monthly_totals(self) -> dict[str, YearExpensesTotals]:
        return self.expenses_by_category_monthly_totals

    def insert_expense(self, expense: ExpenseRecord) -> None:
        if self.filename is None:
            raise ValueError("Expenses file not loaded. Call load_expenses first.")

        self.expenses_records.append(expense)
        self.expenses_by_category_monthly_totals[expense.category][
            expense.date.month - 1
        ] += expense.amount

        with open(self.filename, mode="a", newline="") as file:
            writer = csv.writer(file)
            writer.writerow(
                [
                    expense.timestamp.isoformat(),
                    expense.category,
                    expense.date.isoformat(),
                    f"{expense.amount:.2f}",
                    expense.description,
                ]
            )

    def get_expenses(self) -> list[ExpenseRecord]:
        return self.expenses_records


class Users:
    user_config_file_name = "config.toml"

    def __init__(self):
        self._users_db = {}

    def load(self, db_path: str) -> None:
        if not os.path.exists(db_path):
            raise FileNotFoundError(f"Database path {db_path} does not exist.")

        with os.scandir(db_path) as entries:
            for entry in entries:
                config_path = os.path.join(
                    db_path, entry.name, self.user_config_file_name
                )

                user = self._load_user(config_path)
                if user is None:
                    continue

                self._users_db[user.id] = user
                user.directory = os.path.join(db_path, entry.name)

    def _load_user(self, config_path: str) -> User | None:
        if not os.path.exists(config_path):
            return None

        with open(config_path, "rb") as f:
            config = tomllib.load(f)

        user = User(
            id=config["user"]["username"],
            username=config["user"]["username"],
            password_hash=config["user"]["password_hash"],
        )

        user.full_name = config["user"].get("full_name", None)
        user.currency = config["reports"].get("currency", "")

        return user

    def get(self, username) -> User | None:
        return self._users_db.get(username, None)
