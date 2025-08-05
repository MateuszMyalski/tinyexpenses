import csv
from .file import DbFile, DbCSVReader
from enum import Enum
from collections import defaultdict


class SavingRecord:
    class Columns(Enum):
        CATEGORY = (0, "Category")
        ACCOUNT = (1, "Account")
        BALANCE = (2, "Balance")

        def __init__(self, index: int, label: str):
            self.index = index
            self.label = label

        @classmethod
        def labels(cls):
            return [column.label for column in cls]

    def __init__(self, category: str, account: str, balance: str | float | int) -> None:
        self.category = category.strip()
        self.account = account.strip().lower()
        self.balance = float(balance)

        if self.balance <= 0:
            raise ValueError(
                f"Saving record {category}/{account} cannot have balance <= 0.0"
            )

    def __str__(self):
        return f"{self.category} {self.account} {self.balance}"

    def serialize(self) -> list[str]:
        row = [str()] * len(self.Columns)
        row[self.Columns.ACCOUNT.index] = self.account
        row[self.Columns.CATEGORY.index] = self.category
        row[self.Columns.BALANCE.index] = str(self.balance)

        return row


class Savings:
    def __init__(self, db_file: DbFile) -> None:
        self._db_file = db_file
        self._by_category: dict[str, SavingRecord] = {}
        self._by_account: dict[str, dict[str, SavingRecord]] = defaultdict(dict)
        self._account_totals: dict[str, float] = defaultdict(float)

        self._load_savings()
        self._sum_per_account()

    def _load_savings(self):
        if not self._db_file.exists():
            self._db_file.create()
            return

        with DbCSVReader(self._db_file, SavingRecord.Columns.labels()) as reader:
            for row, line in reader.read():
                try:
                    saving_record = SavingRecord(*line)
                except Exception as reason:
                    raise Exception(
                        f"Cannot parse: {self._db_file.get_file_name()}:{row + 1} - {reason}."
                    )

                if saving_record.category in self._by_category:
                    self._by_category[
                        saving_record.category
                    ].balance += saving_record.balance
                else:
                    self._by_category[saving_record.category] = saving_record

                self._by_account[saving_record.account][saving_record.category] = (
                    saving_record
                )

    def _sum_per_account(self):
        for account, savings in self._by_account.items():
            for saving in savings.values():
                self._account_totals[account] += saving.balance

    def get_savings_account_totals(self) -> dict[str, float]:
        return self._account_totals

    def get_savings_by_account(self) -> dict[str, dict[str, SavingRecord]]:
        return self._by_account

    def get_by_category(self) -> dict[str, SavingRecord]:
        return self._by_category

    def add(self, category: str, account: str | None, value: str | int | float):
        if category in self._by_category:
            return False

        if account is None:
            account = category

        record = SavingRecord(category, category, value)

        self._by_category[category] = record
        self._by_account[account][category] = record
        self._account_totals[account] += record.balance

        return True

    def update(
        self, category: str, account: str | None, value: str | int | float | None
    ):
        if value is None and account is None:
            # Nothing to update
            return

        if value is not None:
            # First check if category already exists if not add
            if self.add(category, account, value):
                return
            else:
                self._update_value(category, float(value))

        if account is not None:
            self._update_account(category, account)

    def _update_value(self, category: str, value: float):
        if value < 0.0:
            raise ValueError(
                "End balance of the savings category cannot be less than 0."
            )

        record = self._by_category[category]
        self._account_totals[record.account] += value - record.balance

        if value == 0.0:
            self._by_account[record.account].pop(record.category)
            self._by_category.pop(record.category)
        else:
            record.balance = value

    def _update_account(self, category: str, account: str):
        if category not in self._by_category:
            return

        record = self._by_category[category]

        self._account_totals[record.account] -= record.balance

        if self._account_totals[record.account] <= 0:
            self._account_totals.pop(record.account)

        self._account_totals[account] += record.balance

        self._by_account[record.account].pop(category)

        record.account = account
        self._by_account[account][category] = record

    def store(self):
        self._db_file.backup()
        self._db_file.erase()

        try:
            with open(self._db_file.get_path(), mode="w", newline="") as file:
                writer = csv.writer(file)
                for savings in self._by_account.values():
                    for record in savings.values():
                        writer.writerow(record.serialize())
        except Exception as e:
            self._db_file.restore()
            raise e
