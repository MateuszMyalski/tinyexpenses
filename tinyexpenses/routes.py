from datetime import datetime
from flask import request, render_template, redirect, url_for
from flask_login import login_required, current_user
from werkzeug import Response
from .expenses_view import expenses_view_year_display, expenses_view_month_display
from .expenses_append import append_expense_form, append_expense
from .expenses_create import create_new_year_report_form, create_new_year_report
from .expenses_edit import edit_expenses_form, edit_expenses
from .extensions import bp
from .auth import auth_display_login_form, auth_authenticate_user, auth_logout
from .categories_create import create_new_categories, create_new_categories_form


@bp.route("/", methods=("GET", "POST"))
def index() -> str | Response:
    if request.method == "GET":
        if current_user.is_authenticated:
            return render_template("dashboard.html", date=datetime.now())
        else:
            return auth_display_login_form()

    if request.method == "POST":
        return auth_authenticate_user()

    return redirect(url_for("main.index"))


@bp.route("/logout", methods=("GET", "POST"))
@login_required
def logout():
    return auth_logout()

@bp.route("/expenses/create/<year>", methods=("GET", "POST"))
@login_required
def expenses_create(year: int):
    if request.method == "POST":
        return create_new_year_report(year)

    if request.method == "GET":
        return create_new_year_report_form(year)

    return render_template("404.html")


@bp.route("/expenses/append", methods=("GET", "POST"))
@login_required
def expenses_append():
    if request.method == "POST":
        return append_expense()

    if request.method == "GET":
        return append_expense_form()

    return render_template("404.html")


@bp.route("/expenses/edit/<year>", methods=("GET", "POST"))
@login_required
def expenses_edit(year: int):

    if request.method == "POST":
        return edit_expenses(year)

    if request.method == "GET":
        return edit_expenses_form(year)

    return render_template("404.html")


@bp.route("/expenses/view/<year>/<month>", methods=("GET", "POST"))
@login_required
def expenses_view_month(year: int, month: int):
    return expenses_view_month_display(year, month)


@bp.route("/expenses/view/", defaults={"year": None}, methods=["GET", "POST"])
@bp.route("/expenses/view/<year>", methods=("GET", "POST"))
@login_required
def expenses_view_year(year: int | None):
    if year is None:
        year = datetime.now().year

    return expenses_view_year_display(year)


@bp.route("/categories/edit", methods=("GET", "POST"))
@login_required
def categories_edit():
    return render_template("404.html")

@bp.route("/categories/create/<year>", methods=("GET", "POST"))
@login_required
def categories_create(year):
    if request.method == "POST":
        return create_new_categories(year)

    if request.method == "GET":
        return create_new_categories_form(year)

    return render_template("404.html")


@bp.route("/settings/view", methods=("GET", "POST"))
@login_required
def settings():
    return render_template("404.html")
