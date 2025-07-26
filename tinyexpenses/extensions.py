from flask_login import LoginManager
from flask import Blueprint
from .models import Users

login_manager = LoginManager()

bp = Blueprint('main', __name__)

users_db = Users()


def format_number(value):
    """Format a number with commas as thousand separators."""
    return "{:0,.2f}".format(value)
