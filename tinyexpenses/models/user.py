import os
import tomllib
import tomli_w
import flask_login
from .file import DbFile
from werkzeug.security import check_password_hash, generate_password_hash


class Config:
    CONFIG_FILE_NAME = "config.toml"

    def __init__(self, db_file) -> None:
        self._db_file = DbFile(os.path.join(db_file, Config.CONFIG_FILE_NAME))
        self._data = self._load()

        if self._data.get("user", None) is None:
            self._data["user"] = {
                "username": "",
                "full_name": "",
                "password_hash": "",
            }
            self._save()

        self._base_config = self._data["user"]

    def _load(self) -> dict:
        if not self._db_file.exists():
            raise FileNotFoundError(f"Missing config: {self._db_file.get_path()}")
        with open(self._db_file.get_path(), "rb") as f:
            return tomllib.load(f)

    def _save(self) -> None:
        with open(self._db_file.get_path(), "wb") as f:
            tomli_w.dump(self._data, f)

    def set_username(self, username):
        self._base_config["username"] = username
        self._save()

    def get_username(self) -> str:
        return self._base_config["username"]

    def get_full_name(self) -> str:
        return self._data["user"].get("full_name", "")

    def set_full_name(self, full_name: str) -> None:
        self._base_config["full_name"] = full_name
        self._save()

    def get_password_hash(self) -> str:
        return self._base_config["password_hash"]

    def check_password(self, password: str) -> bool:
        return check_password_hash(self.get_password_hash(), password)

    def set_password(self, password: str):
        self._base_config["password_hash"] = generate_password_hash(password)
        self._save()

    def change_password(self, current_password: str, new_password: str) -> bool:
        if not self.check_password(current_password):
            return False

        self._base_config["password_hash"] = generate_password_hash(new_password)
        self._save()

        return True

    @staticmethod
    def config_file_exists(path) -> bool:
        user_cfg_file = DbFile(os.path.join(path, Config.CONFIG_FILE_NAME))

        return user_cfg_file.exists()

    @staticmethod
    def config_file_create_empty(path):
        if not Config.config_file_exists(path):
            DbFile(os.path.join(path, Config.CONFIG_FILE_NAME)).create()


class User(flask_login.UserMixin):
    def __init__(self, id, config):
        self.id = id
        self._config = config
        self._user_directory = ""
        self._app_path = ""

    def set_user_directory(self, directory):
        raise NotImplementedError()

    @property
    def username(self):
        return self._config.get_username()

    @property
    def full_name(self):
        return self._config.get_full_name()

    def check_password(self, password: str) -> bool:
        return self._config.check_password(password)

    def change_password(self, current: str, new: str) -> bool:
        return self._config.change_password(current, new)

    def set_full_name(self, name: str):
        self._config.set_full_name(name)

    def get_id(self):
        return self.id
