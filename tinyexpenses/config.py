import os


class Config:
    SECRET_KEY = os.environ.get("SECRET_KEY", "unsafe-default-key")
    ACCOUNTS_DB_DIRECTORY_PATH = os.environ.get(
        "ACCOUNTS_DB_DIRECTORY_PATH", "accounts"
    )

class ProductionHTTPConfig(Config):
    DEBUG = False
    TESTING = False

class ProductionHTTPSConfig(ProductionHTTPConfig):
    SESSION_COOKIE_SECURE = True
    REMEMBER_COOKIE_SECURE = True

class DevelopmentConfig(Config):
    DEBUG = True
    TESTING = True
