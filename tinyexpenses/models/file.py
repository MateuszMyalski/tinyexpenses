import os
import csv
from contextlib import AbstractContextManager


class DbFile:
    BACKUP_FILE_NAME_SUFFIX = ".bak"

    def __init__(self, file_path: str):
        self._dir = os.path.dirname(file_path)

        self._file_name = os.path.basename(file_path)
        self._file_path = os.path.join(self._dir, self._file_name)
        self._backup_file_path = os.path.join(
            self._dir, self._file_name + self.BACKUP_FILE_NAME_SUFFIX
        )

    def exists(self) -> bool:
        return os.path.exists(self._file_path)

    def create(self):
        if self.exists():
            raise FileExistsError(f"File {self._file_path} already exists.")

        os.makedirs(os.path.dirname(self._file_path), exist_ok=True)

        with open(self._file_path, mode="w", newline="") as _:
            pass

    def erase(self):
        if not os.path.exists(self._file_path):
            raise FileNotFoundError(f"Provided path does not exist {self._file_path}.")

        with open(self._file_path, mode="r+", newline="") as file:
            file.truncate(0)

    def backup(self):
        self.copy_to(self._backup_file_path)

    def restore(self):
        if not os.path.exists(self._backup_file_path):
            raise FileNotFoundError(
                f"Backup file for {self._file_path} does not exist."
            )

        try:
            self.erase()
        except FileNotFoundError:
            pass

        self.copy_from(self._backup_file_path)

    def copy_from(self, src_file):
        if not os.path.exists(src_file):
            raise FileNotFoundError(f"Provided path does not exist {src_file}.")

        with (
            open(src_file, "rb") as src,
            open(self._file_path, "wb") as dst,
        ):
            dst.write(src.read())

    def copy_to(self, dst_file):
        if not os.path.exists(self._file_path):
            raise FileNotFoundError(f"Provided path does not exist {self._file_path}.")
        with (
            open(self._file_path, "rb") as src,
            open(dst_file, "wb") as dst,
        ):
            dst.write(src.read())

    def get_path(self) -> str:
        return os.path.join(self._dir, self._file_name)

    def get_file_name(self) -> str:
        return self._file_name


class DbCSVReader(AbstractContextManager):
    def __init__(self, db_file: DbFile, columns: list):
        self._db_file = db_file
        self._columns = columns

    def __enter__(self):
        self._file = open(self._db_file.get_path(), mode="r", newline="")
        self._reader = csv.reader(self._file)
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        self._file.close()

    def read(self):
        for row, line in enumerate(self._reader):
            if len(line) == 0:
                continue

            if len(line) != len(self._columns):
                raise Exception(
                    f"Cannot parse: {self._db_file.get_file_name()}:{row + 1} -  Read {len(line)}/{len(self._columns)} columns."
                )

            yield (row, line)


class DbCSVWriter(AbstractContextManager):
    def __init__(self, db_file: DbFile, columns: list, append_mode=True):
        self._db_file = db_file
        self._columns = columns

        if append_mode:
            self._mode = "a"
        else:
            self._mode = "w"

    def __enter__(self):
        self.ensure_trailing_newline()
        self._file = open(self._db_file.get_path(), mode=self._mode, newline="")
        self._writer = csv.writer(self._file)
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        self._file.close()

    def ensure_trailing_newline(self):
        with open(self._db_file.get_path(), "rb+") as f:
            f.seek(-1, os.SEEK_END)
            if f.read(1) != b"\n":
                f.write(b"\n")

    def write(self, row: list):
        if len(row) != len(self._columns):
            raise Exception(
                f"Cannot write: {self._db_file.get_file_name()} - Got {len(row)} columns, expected {len(self._columns)}."
            )
        self._writer.writerow(row)
