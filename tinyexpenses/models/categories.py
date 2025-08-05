import csv
from enum import Enum
from .file import DbFile, DbCSVReader, DbCSVWriter
from collections import defaultdict


class CategoryType(Enum):
    SAVINGS = "Savings"
    WANTS = "Wants"
    NEEDS = "Needs"
    INCOME = "Income"
    INITIAL_BALANCE_LABEL = "Initial Balance"


class CategoryRecord:
    class Columns(Enum):
        CATEGORY = (0, "Category")
        CATEGORY_TYPE = (1, "Category type")

        def __init__(self, index: int, label: str):
            self.index = index
            self.label = label

        @classmethod
        def labels(cls):
            return [column.label for column in cls]

    def __init__(self, category: str, category_type: str | CategoryType):
        if isinstance(category_type, CategoryType):
            self.category_type = category_type
        elif isinstance(category_type, str):
            try:
                self.category_type = CategoryType(category_type.strip().title())
            except ValueError:
                valid_cat_types = [cat.value for cat in CategoryType]
                raise ValueError(
                    f"Invalid category type: {category_type}. Valid only {valid_cat_types}"
                )
        else:
            raise TypeError("category_type must be a str or CategoryType.")

        self.category = category.strip()

    def __str__(self):
        return f"{self.category} {self.category_type.value}"

    def __iter__(self):
        return iter((self.category, self.category_type.value))

    def serialize(self) -> list[str]:
        row = [str()] * len(self.Columns)
        row[self.Columns.CATEGORY_TYPE.index] = self.category_type.value
        row[self.Columns.CATEGORY.index] = self.category

        return row


class YearCategories:
    def __init__(self, db_file: DbFile):
        self._db_file = db_file
        self._by_category: dict[str, CategoryRecord] = {}
        self._by_category_type: dict[CategoryType, dict[str, CategoryRecord]] = (
            defaultdict(dict)
        )

        self._load_categories()

    def _load_categories(self) -> None:
        if not self._db_file.exists():
            raise FileNotFoundError(f"File {self._db_file.get_path()} does not exist.")

        with DbCSVReader(self._db_file, CategoryRecord.Columns.labels()) as reader:
            for row, line in reader.read():
                try:
                    category_record = CategoryRecord(*line)
                except Exception as reason:
                    raise Exception(
                        f"Cannot parse: {self._db_file.get_file_name()}:{row + 1} - {reason}."
                    )

                self._by_category[category_record.category] = category_record
                self._by_category_type[category_record.category_type][
                    category_record.category
                ] = category_record

    def get_categories(self) -> list[CategoryRecord]:
        return list(self._by_category.values())

    def __getitem__(self, key: CategoryType | str):
        if isinstance(key, CategoryType):
            pass
        elif isinstance(key, str):
            key = CategoryType(key.title())
        else:
            raise TypeError("key must be a str or CategoryType")

        return list(
            map(
                lambda r: r.category,
                self._by_category_type[key].values(),
            )
        )

    def insert_category(self, record: CategoryRecord):
        if record.category in self._by_category:
            return

        self._by_category[record.category] = record
        self._by_category_type[record.category_type][record.category] = record

        YearCategories.store(self._db_file, list(self._by_category.values()))

    @staticmethod
    def store(db_file: DbFile, categories: list[CategoryRecord]) -> None:
        db_file.backup()
        db_file.erase()

        try:
            with open(db_file.get_path(), mode="w", newline="") as file:
                writer = csv.writer(file)
                for category in categories:
                    writer.writerow(category.serialize())

        except Exception as e:
            db_file.restore()
            raise e
