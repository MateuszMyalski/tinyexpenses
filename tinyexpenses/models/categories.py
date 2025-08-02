import csv
from enum import Enum
from .file import DbFile


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

    def __init__(self, category: str, category_type: str | CategoryType):
        if isinstance(category_type, CategoryType):
            self.category_type = category_type
        elif isinstance(category_type, str):
            try:
                self.category_type = CategoryType(category_type.title())
            except ValueError:
                valid_cat_types = [cat.value for cat in CategoryType]
                raise ValueError(f"Invalid category type: {category_type}. Valid only {valid_cat_types}")
        else:
            raise TypeError("category_type must be a str or CategoryType.")

        self.category = category

    def __str__(self):
        return f"{self.category} {self.category_type.value}"

    def __iter__(self):
        return iter((self.category, self.category_type.value))


class YearCategories:
    BACKUP_FILE_SUFFIX = ".bak"

    def __init__(self, db_file: DbFile):
        self.db_file = db_file
        self.categories_records: list[CategoryRecord] = list()

        self._load_categories()

    def _load_categories(self) -> None:
        if not self.db_file.exists():
            raise FileNotFoundError(f"File {self.db_file.get_path()} does not exist.")

        with open(self.db_file.get_path(), mode="r", newline="") as file:
            reader = csv.reader(file)

            for row, record in enumerate(reader):
                # Empty line
                if len(record) == 0:
                    continue

                if len(record) != len(CategoryRecord.Columns):
                    raise Exception(
                        f"Cannot parse: {file.name}:{row + 1} -  Read {len(record)} columns. Expected {len(CategoryRecord.Columns)} elements."
                    )

                category: str = record[CategoryRecord.Columns.CATEGORY.index].strip()
                cat_type = (
                    record[CategoryRecord.Columns.CATEGORY_TYPE.index].strip().title()
                )

                self.categories_records.append(CategoryRecord(category, cat_type))

    def get_categories(self) -> list[CategoryRecord]:
        return self.categories_records

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
                filter(lambda r: r.category_type == key, self.categories_records),
            )
        )

    def __setitem__(self, key: CategoryType | str, value: str) -> None:
        if isinstance(key, CategoryType):
            category_type = key
        elif isinstance(key, str):
            category_type = CategoryType(key.title())
        else:
            raise TypeError("key must be a str or CategoryType")

        self.categories_records.append(CategoryRecord(value, category_type))

    @staticmethod
    def insert_category(
        db_file: DbFile, categories: CategoryRecord | list[CategoryRecord]
    ):
        if not isinstance(categories, list):
            categories = [categories]

        with open(db_file.get_path(), mode="a", newline="") as file:
            writer = csv.writer(file)
            for category in categories:
                row = [str()] * len(CategoryRecord.Columns)
                row[CategoryRecord.Columns.CATEGORY_TYPE.index] = category.category_type.value
                row[CategoryRecord.Columns.CATEGORY.index] = category.category

                writer.writerow(row)

    @staticmethod
    def store(db_file: DbFile, categories: list[CategoryRecord]) -> None:
        db_file.backup()
        db_file.erase()

        try:
            YearCategories.insert_category(db_file, categories)
        except Exception as e:
            db_file.restore()
            raise e
