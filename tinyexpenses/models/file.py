import os


class DbFile:
    BACKUP_FILE_NAME_SUFFIX = ".bak"

    def __init__(self, file_path: str):
        self.dir = os.path.dirname(file_path)

        self.file_name = os.path.basename(file_path)
        self.file_path = os.path.join(self.dir, self.file_name)
        self.backup_file_path = os.path.join(
            self.dir, self.file_name + self.BACKUP_FILE_NAME_SUFFIX
        )

    def exists(self) -> bool:
        return os.path.exists(self.file_path)

    def create(self):
        if self.exists():
            raise FileExistsError(f"File {self.file_path} already exists.")

        os.makedirs(os.path.dirname(self.file_path), exist_ok=True)

        with open(self.file_path, mode="w", newline="") as _:
            pass

    def erase(self):
        if not os.path.exists(self.file_path):
            raise FileNotFoundError(f"Provided path does not exist {self.file_path}.")

        with open(self.file_path, mode="r+", newline="") as file:
            file.truncate(0)

    def backup(self):
        self.copy_to(self.backup_file_path)

    def restore(self):
        if not os.path.exists(self.backup_file_path):
            raise FileNotFoundError(f"Backup file for {self.file_path} does not exist.")

        try:
            self.erase()
        except FileNotFoundError:
            pass

        self.copy_from(self.backup_file_path)

    def copy_from(self, src_file):
        if not os.path.exists(src_file):
            raise FileNotFoundError(f"Provided path does not exist {src_file}.")

        with (
            open(src_file, "rb") as src,
            open(self.file_path, "wb") as dst,
        ):
            dst.write(src.read())

    def copy_to(self, dst_file):
        if not os.path.exists(self.file_path):
            raise FileNotFoundError(f"Provided path does not exist {self.file_path}.")
        with (
            open(self.file_path, "rb") as src,
            open(dst_file, "wb") as dst,
        ):
            dst.write(src.read())

    def get_path(self) -> str:
        return os.path.join(self.dir, self.file_name)
