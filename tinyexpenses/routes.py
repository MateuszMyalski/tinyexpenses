from datetime import datetime
from flask import request, render_template, redirect, url_for
from flask_login import login_required, current_user
from .expenses_view import expenses_view_year, expenses_view_month

from .extensions import bp
from .auth import auth_display_login_form, auth_authenticate_user, auth_logout

@bp.route("/",  methods=("GET", "POST"))
def index():
    if request.method == "GET":
        if current_user.is_authenticated:
            return render_template("dashboard.html", date=datetime.now())
        else:
            return auth_display_login_form()

    if request.method == 'POST':
        return auth_authenticate_user()

    return redirect(url_for("main.index"))

@bp.route("/logout", methods=("GET", "POST"))
@login_required
def logout():
    return auth_logout()

@bp.route("/expenses/log/append", methods=("GET", "POST"))
@login_required
def expenses_log_append():
    return render_template("404.html")

@bp.route("/expenses/log/edit", methods=("GET", "POST"))
@login_required
def expenses_log_edit():
    return render_template("404.html")

@bp.route("/view/expenses/<year>/<month>", methods=("GET", "POST"))
@login_required
def view_month(year: int, month: int):
    return expenses_view_month(year, month)

@bp.route("/view/expenses", defaults={"year": None}, methods=["GET", "POST"])
@bp.route("/view/expenses/<year>", methods=("GET", "POST"))
@login_required
def view_year(year: int | None):
    if year is None:
        year = datetime.now().year
    return expenses_view_year(year)

@bp.route("/groups/edit", methods=("GET", "POST"))
@login_required
def groups_edit():
    return render_template("404.html")

@bp.route("/settings/view", methods=("GET", "POST"))
@login_required
def settings():
    return render_template("404.html")