import json
from flask import (
    render_template,
    redirect,
    url_for,
    flash,
    get_flashed_messages,
    jsonify,
    request,
)
from flask_login import current_user
from flask_wtf import FlaskForm
from wtforms import (
    SubmitField,
    DateField,
    FloatField,
    TextAreaField,
    SelectField,
    validators,
    ValidationError,
)

from tinyexpenses.models.accounts import User
from .models.categories import YearCategories, CategoryRecord
from .models.expenses import YearExpensesReport, ExpenseRecord
from .extensions import users_db
from datetime import datetime, date


EXPENSE_APPEND_FLASH_LABEL = "append_expense_info"


class AppendExpenseForm(FlaskForm):
    @staticmethod
    def validate_date(form, field):
        if field.data.year != date.today().year:
            raise ValidationError("Only dates from the current year are allowed.")

    category = SelectField(
        "Category", [validators.DataRequired()], choices=[], coerce=str
    )
    expense_date = DateField(
        "Date",
        [validators.DataRequired(message="Date is required."), validate_date],
        default=datetime.today,
    )
    amount = FloatField(
        "Amount", [validators.DataRequired(message="Amount is required.")]
    )
    description = TextAreaField(
        "Description", [validators.Optional(strip_whitespace=True)]
    )
    submit = SubmitField("Submit")

    def populate_category_choices(self, categories: list[CategoryRecord]):
        self.category.choices = list(
            map(lambda item: (item.category, item.category), categories)
        )


def expenses_append_get():
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    # Currently we support adding expenses only to current year
    year_categories_file = requested_user.get_categories_file(datetime.now().year)
    year_categories = YearCategories(year_categories_file)

    flashed_info = [
        ("info", msg) for msg in get_flashed_messages(False, EXPENSE_APPEND_FLASH_LABEL)
    ]
    form = AppendExpenseForm()
    form.populate_category_choices(year_categories.get_categories())

    return render_template(
        "expenses_append.html",
        form=form,
        infos=flashed_info,
        current_year=datetime.now().year,
    )


def expenses_append_post():
    requested_user: User | None = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    # Currently we support adding expenses only to current year
    year_categories_file = requested_user.get_categories_file(datetime.now().year)
    year_expenses_file = requested_user.get_expenses_report_file(datetime.now().year)
    year_categories = YearCategories(year_categories_file)

    form = AppendExpenseForm()
    form.populate_category_choices(year_categories.get_categories())

    if not form.validate_on_submit():
        return render_template(
            "expenses_append.html",
            form=form,
            infos=[("error", "Request could not be validated.")],
            current_year=datetime.now().year,
        )

    try:
        expense = ExpenseRecord(
            timestamp=datetime.now().isoformat(),
            category=form.category.data,
            expense_date=form.expense_date.data,
            amount=form.amount.data,
            description=form.description.data,
        )

        YearExpensesReport.insert_expense(year_expenses_file, expense)
    except FileNotFoundError:
        return redirect(
            url_for("main.expenses_create", year=form.expense_date.data.year)
        )

    flash("Request completed.", EXPENSE_APPEND_FLASH_LABEL)
    return redirect(url_for("main.expenses_append"))


def expenses_append_api_put(username):
    if request.headers.get("Content-type", "") != "application/json":
        return jsonify({"status": "Content-type nor supported."}), 400

    requested_user = users_db.get(username)

    if requested_user is None:
        return jsonify({"status": "Unauthorized"}), 401

    try:
        user_request_data = json.loads(request.data.decode())

        expense = ExpenseRecord(
            timestamp=datetime.now().isoformat(),
            amount=user_request_data["amount"],
            category=user_request_data["category"],
            expense_date=user_request_data.get("expense_date", datetime.now().date()),
            description=user_request_data.get("description", ""),
        )
    except Exception as e:
        return jsonify(
            {
                "status": "Could not parse request.",
                "received_content": f"{request.data.decode()}",
                "exception:" : f"{e}"
            }
        ), 400

    # We support adding expences only to current year
    if expense.expense_date.year != datetime.now().date().year:
        return jsonify(
            {"status": f"Cannot add expense for year {expense.expense_date.year}."}
        ), 501

    try:
        year_categories_file = requested_user.get_categories_file(
            expense.expense_date.year
        )
        year_categories = YearCategories(year_categories_file)

        available_categories = list(
            map(lambda item: item.category, year_categories.get_categories())
        )
    except Exception:
        return jsonify(
            {
                "status": f"Could not load categories for given year {expense.expense_date.year}."
            }
        ), 500

    if expense.category not in available_categories:
        return jsonify(
            {
                "status": f"Category does exists for given year {expense.expense_date.year}"
            }
        )
    try:
        year_expenses_file = requested_user.get_expenses_report_file(
            expense.expense_date.year
        )
    except Exception:
        return jsonify(
            {
                "status": f"Could not load expenses for given year {expense.expense_date.year}"
            }
        ), 500

    try:
        YearExpensesReport.insert_expense(year_expenses_file, expense)
    except Exception:
        return jsonify(
            {
                "status": f"Could not append expense file for given year {expense.expense_date.year}."
            }
        ), 500

    return jsonify({"status": "Ok"}), 200
