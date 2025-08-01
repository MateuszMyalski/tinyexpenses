from flask import Flask
from flask_login import LoginManager
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address
from flask import Blueprint
from .models.accounts import Users

app = Flask(__name__, instance_relative_config=True)

login_manager = LoginManager()

bp = Blueprint("main", __name__)

users_db = Users()

limiter = Limiter(
    get_remote_address,
    app=app,
    default_limits=["50 per minute"],
    storage_uri="memory://",
    strategy="fixed-window"
)


def format_number(value):
    """Format a number with commas as thousand separators."""
    return "{:0,.2f}".format(value)
