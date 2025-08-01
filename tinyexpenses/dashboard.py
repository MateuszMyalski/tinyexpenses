from flask import render_template
from flask_login import current_user
from datetime import datetime
from .models.accounts import User
from .extensions import users_db
from .auth import auth_display_login_form


def dashboard_get():
    if not current_user.is_authenticated:
        return auth_display_login_form()

    requested_user: User | None = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    return render_template(
        "dashboard.html", date=datetime.now(), full_name=requested_user.full_name
    )
