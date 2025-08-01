from datetime import datetime
from flask import request, render_template, redirect, url_for, jsonify
from flask_login import login_required
from functools import wraps
from werkzeug import Response
from .expenses_view import expenses_view_year_display, expenses_view_month_display
from .expenses_append import append_expense_form, append_expense, append_expense_api
from .expenses_create import create_new_year_report_form, create_new_year_report
from .expenses_edit import edit_expenses_form, edit_expenses
from .extensions import bp, users_db, login_manager
from .auth import auth_authenticate_user, auth_logout
from .categories_create import create_new_categories, create_new_categories_form
from .categories_edit import edit_categories_form, edit_categories
from .account import account_handle_change, account_handle_change_form
from .token import verify_user_token
from .dashboard import dashboard_get


def handle_uncaught_exceptions(f):
    @wraps(f)
    def wrapper(*args, **kwargs):
        try:
            return f(*args, **kwargs)
        except Exception as e:
            return render_template("error.html", message=str(e))

    return wrapper


def api_key_required(view_function):
    @wraps(view_function)
    def wrapper(*args, **kwargs):
        x_api_key = request.headers.get("X-API-Key")
        user = users_db.get(kwargs["username"])

        if user is None:
            return jsonify({"status": "Unauthorized"}), 401

        if x_api_key is None:
            return jsonify({"status": "Unauthorized"}), 401

        if verify_user_token(x_api_key) is None:
            return jsonify({"status": "Unauthorized"}), 401

        return view_function(*args, **kwargs)

    return wrapper


@login_manager.unauthorized_handler
def handle_needs_login():
    return redirect(url_for("main.index"))


@bp.route("/", methods=("GET", "POST"))
@handle_uncaught_exceptions
def index() -> str | Response:
    if request.method == "GET":
        return dashboard_get()

    if request.method == "POST":
        return auth_authenticate_user()

    return redirect(url_for("main.index"))


@bp.route("/logout", methods=("GET", "POST"))
@handle_uncaught_exceptions
@login_required
def logout():
    return auth_logout()


@bp.route("/expenses/create/<year>", methods=("GET", "POST"))
@handle_uncaught_exceptions
@login_required
def expenses_create(year: int):
    if request.method == "POST":
        return create_new_year_report(year)

    if request.method == "GET":
        return create_new_year_report_form(year)

    return render_template("404.html")


@bp.route("/expenses/append", methods=("GET", "POST"))
@handle_uncaught_exceptions
@login_required
def expenses_append():
    if request.method == "POST":
        return append_expense()

    if request.method == "GET":
        return append_expense_form()

    return render_template("404.html")


@bp.route("/v1.0/<username>/expenses/append", methods=("PUT", "POST"))
@api_key_required
def expenses_append_api(username):
    return append_expense_api(username)


@bp.route("/expenses/edit/<year>", methods=("GET", "POST"))
@handle_uncaught_exceptions
@login_required
def expenses_edit(year: int):
    if request.method == "POST":
        return edit_expenses(year)

    if request.method == "GET":
        return edit_expenses_form(year)

    return render_template("404.html")


@bp.route("/expenses/view/<year>/<month>", methods=("GET", "POST"))
@handle_uncaught_exceptions
@login_required
def expenses_view_month(year: int, month: int):
    return expenses_view_month_display(year, month)


@bp.route("/expenses/view/", defaults={"year": None}, methods=["GET", "POST"])
@bp.route("/expenses/view/<year>", methods=("GET", "POST"))
@handle_uncaught_exceptions
@login_required
def expenses_view_year(year: int | None):
    if year is None:
        year = datetime.now().year

    return expenses_view_year_display(year)


@bp.route("/categories/edit/<year>", methods=("GET", "POST"))
@handle_uncaught_exceptions
@login_required
def categories_edit(year: int):
    if request.method == "POST":
        return edit_categories(year)

    if request.method == "GET":
        return edit_categories_form(year)

    return render_template("404.html")


@bp.route("/categories/create/<year>", methods=("GET", "POST"))
@handle_uncaught_exceptions
@login_required
def categories_create(year):
    if request.method == "POST":
        return create_new_categories(year)

    if request.method == "GET":
        return create_new_categories_form(year)

    return render_template("404.html")


@bp.route("/account/view", methods=("GET", "POST"))
@handle_uncaught_exceptions
@login_required
def account():
    if request.method == "POST":
        return account_handle_change()

    if request.method == "GET":
        return account_handle_change_form()

    return render_template("404.html")
