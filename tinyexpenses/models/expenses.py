import datetime
import dateutil
import csv
import calendar
from enum import Enum
from dataclasses import dataclass, field
from datetime import date, datetime
from .categories import CategoryType

from .file import DbFile


@dataclass
class YearExpensesTotals:
    totals: list[float] = field(
        default_factory=lambda: [0.0] * (len(calendar.month_name) - 1)
    )

    def __post_init__(self):
        if len(self.totals) != (len(calendar.month_name) - 1):
            raise ValueError(
                f"Totals must have exactly {len(calendar.month_name) - 1} elements."
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
        return iter(self.totals)


class ExpenseRecord:
    class Columns(Enum):
        TIMESTAMP = (0, "Timestamp")
        CATEGORY = (1, "Category")
        EXPENSE_DATE = (2, "Expense date")
        AMOUNT = (3, "Amount")
        DESCRIPTION = (4, "Description")

        def __init__(self, index: int, label: str):
            self.index = index
            self.label = label

    def __init__(
        self,
        timestamp: str | datetime,
        category: str,
        expense_date: str | date,
        amount: str | float,
        description: str,
    ):
        if isinstance(timestamp, datetime):
            self.timestamp = timestamp
        elif isinstance(timestamp, str):
            self.timestamp = dateutil.parser.parse(timestamp)
        else:
            raise TypeError("Invalid type of timestamp.")

        if isinstance(expense_date, date):
            self.expense_date = expense_date
        elif isinstance(expense_date, str):
            self.expense_date = dateutil.parser.parse(expense_date).date()
        else:
            raise TypeError("Invalid type of date.")

        self.description = description

        if isinstance(amount, float):
            self.amount = amount
        elif isinstance(amount, str):
            self.amount = float(amount.replace(",", "."))
        else:
            raise TypeError("Invalid type of amount.")

        self.category = category

    def __str__(self) -> str:
        return f"{self.timestamp} | {self.category} | {self.expense_date} | {self.amount} | {self.description}"

    def __iter__(self):
        return iter(
            (
                self.timestamp,
                self.category,
                self.expense_date,
                self.amount,
                self.description,
            )
        )


class YearExpensesReport:
    def __init__(self, db_file: DbFile):
        self.db_file: DbFile = db_file
        self.expenses_records: list[ExpenseRecord] = list()
        self.expenses_by_category: dict[str, list[ExpenseRecord]] = dict()
        self.expenses_by_category_monthly_totals: dict[str, YearExpensesTotals] = dict()
        self.initial_balance: float = 0.0

        self._load_expenses()
        self._sum_expenses_by_category_per_month()

    def _load_expenses(self) -> None:
        if not self.db_file.exists():
            raise FileNotFoundError(f"File {self.db_file.get_path()} does not exist.")

        with open(self.db_file.get_path(), mode="r", newline="") as file:
            reader = csv.reader(file)

            for row, record in enumerate(reader):
                # Empty line
                if len(record) == 0:
                    continue

                if len(record) != len(ExpenseRecord.Columns):
                    raise Exception(
                        f"Cannot parse: {file.name}:{row + 1} - Read {len(record)} columns. Expected {len(ExpenseRecord.Columns)} elements."
                    )

                try:
                    expense = ExpenseRecord(
                        timestamp=record[ExpenseRecord.Columns.TIMESTAMP.index],
                        category=record[ExpenseRecord.Columns.CATEGORY.index],
                        expense_date=record[ExpenseRecord.Columns.EXPENSE_DATE.index],
                        amount=record[ExpenseRecord.Columns.AMOUNT.index],
                        description=record[ExpenseRecord.Columns.DESCRIPTION.index],
                    )
                except Exception as reason:
                    raise Exception(f"Cannot parse: {file.name}:{row + 1} - {reason}.")

                if expense.category == CategoryType.INITIAL_BALANCE_LABEL.value:
                    self.initial_balance = expense.amount

                self.expenses_records.append(expense)
                self.expenses_by_category.setdefault(expense.category, []).append(
                    expense
                )

    def _sum_expenses_by_category_per_month(self) -> None:
        for category, expenses in self.expenses_by_category.items():
            for expense in expenses:
                self.expenses_by_category_monthly_totals.setdefault(
                    category, YearExpensesTotals()
                ).totals[expense.expense_date.month - 1] += expense.amount

    def get_expenses_by_category_monthly_totals(self) -> dict[str, YearExpensesTotals]:
        return self.expenses_by_category_monthly_totals

    def get_expenses(self) -> list[ExpenseRecord]:
        return self.expenses_records

    @staticmethod
    def insert_expense(
        db_file: DbFile,
        expenses: ExpenseRecord | list[ExpenseRecord],
    ) -> None:
        if not isinstance(expenses, list):
            expenses = [expenses]

        if not db_file.exists():
            raise FileNotFoundError("Expenses file does not exists.")

        with open(db_file.get_path(), mode="a", newline="") as file:
            writer = csv.writer(file)
            for expense in expenses:
                row = [str()] * len(ExpenseRecord.Columns)
                row[ExpenseRecord.Columns.AMOUNT.index] = f"{expense.amount:.2f}"
                row[ExpenseRecord.Columns.CATEGORY.index] = expense.category
                row[ExpenseRecord.Columns.TIMESTAMP.index] = expense.timestamp.strftime(
                    "%Y-%m-%d %H:%M:%S"
                )
                row[ExpenseRecord.Columns.DESCRIPTION.index] = expense.description
                row[ExpenseRecord.Columns.EXPENSE_DATE.index] = (
                    expense.expense_date.strftime("%Y-%m-%d")
                )

                writer.writerow(row)

    @staticmethod
    def store(db_file: DbFile, expenses: list[ExpenseRecord]) -> None:
        db_file.backup()
        db_file.erase()

        try:
            YearExpensesReport.insert_expense(db_file, expenses)
        except Exception as e:
            db_file.restore()
            raise e
