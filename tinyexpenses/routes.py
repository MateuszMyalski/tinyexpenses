from datetime import datetime
from flask import request, render_template, redirect, url_for, jsonify
from flask_login import login_required
from functools import wraps
from werkzeug import Response
from .expenses_view import expenses_view_year_get, expenses_view_month_get
from .expenses_append import expenses_append_get, expenses_append_post, expenses_append_api_put
from .expenses_create import expenses_create_get, expenses_create_post
from .expenses_edit import expenses_edit_get, expenses_edit_post
from .extensions import bp, users_db, login_manager, csrf
from .auth import auth_authenticate_post, auth_logout
from .categories_create import categories_create_post, categories_create_get
from .categories_edit import categories_edit_get, categories_edit_post
from .account import account_post, account_get
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
        return auth_authenticate_post()

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
        return expenses_create_post(year)

    if request.method == "GET":
        return expenses_create_get(year)

    return render_template("404.html")


@bp.route("/expenses/append", methods=("GET", "POST"))
@handle_uncaught_exceptions
@login_required
def expenses_append():
    if request.method == "POST":
        return expenses_append_post()

    if request.method == "GET":
        return expenses_append_get()

    return render_template("404.html")


@bp.route("/api/v1/<username>/expenses/append", methods=("PUT", "POST"))
@api_key_required
@csrf.exempt
def expenses_append_api(username):
    return expenses_append_api_put(username)


@bp.route("/expenses/edit/<year>", methods=("GET", "POST"))
@handle_uncaught_exceptions
@login_required
def expenses_edit(year: int):
    if request.method == "POST":
        return expenses_edit_post(year)

    if request.method == "GET":
        return expenses_edit_get(year)

    return render_template("404.html")


@bp.route("/expenses/view/<year>/<month>", methods=("GET", "POST"))
@handle_uncaught_exceptions
@login_required
def expenses_view_month(year: int, month: int):
    return expenses_view_month_get(year, month)


@bp.route("/expenses/view/", defaults={"year": None}, methods=["GET", "POST"])
@bp.route("/expenses/view/<year>", methods=("GET", "POST"))
@handle_uncaught_exceptions
@login_required
def expenses_view_year(year: int | None):
    if year is None:
        year = datetime.now().year

    return expenses_view_year_get(year)


@bp.route("/categories/edit/<year>", methods=("GET", "POST"))
@handle_uncaught_exceptions
@login_required
def categories_edit(year: int):
    if request.method == "POST":
        return categories_edit_post(year)

    if request.method == "GET":
        return categories_edit_get(year)

    return render_template("404.html")


@bp.route("/categories/create/<year>", methods=("GET", "POST"))
@handle_uncaught_exceptions
@login_required
def categories_create(year):
    if request.method == "POST":
        return categories_create_post(year)

    if request.method == "GET":
        return categories_create_get(year)

    return render_template("404.html")


@bp.route("/account/view", methods=("GET", "POST"))
@handle_uncaught_exceptions
@login_required
def account():
    if request.method == "POST":
        return account_post()

    if request.method == "GET":
        return account_get()

    return render_template("404.html")
