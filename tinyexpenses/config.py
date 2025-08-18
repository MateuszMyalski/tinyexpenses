import os
from datetime import timedelta

class Config:
    SECRET_KEY = os.environ.get("SECRET_KEY", "unsafe-default-key")
    ACCOUNTS_DB_DIRECTORY_PATH = os.environ.get(
        "ACCOUNTS_DB_DIRECTORY_PATH", "accounts"
    )
    REMEMBER_COOKIE_DURATION = timedelta(days=30)


class ProductionHTTPConfig(Config):
    DEBUG = False
    TESTING = False


class ProductionHTTPSConfig(ProductionHTTPConfig):
    SESSION_COOKIE_SECURE = True
    REMEMBER_COOKIE_SECURE = True


class DevelopmentConfig(Config):
    DEBUG = True
    TESTING = True
    WTF_CSRF_SSL_STRICT = False
    SESSION_COOKIE_SECURE = False
    SESSION_COOKIE_SAMESITE = None
    SESSION_COOKIE_DOMAIN = False
