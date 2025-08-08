import json
from flask import (
    render_template,
    redirect,
    url_for,
    flash,
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
from .models.categories import YearCategories, CategoryType
from .models.expenses import ExpenseRecord
from .extensions import users_db
from .models.flash import FlashType, flash_collect
from datetime import datetime, date


class AppendExpenseForm(FlaskForm):
    @staticmethod
    def validate_date(form, field):
        if field.data.year != date.today().year:
            raise ValidationError("Only dates from the current year are allowed.")

    category = SelectField(
        "Category", [validators.DataRequired()], choices={}, coerce=str
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

    def populate_category_choices(self, year_categories: YearCategories):
        self.category.choices = {}
        for category_type, categories in year_categories._by_category_type.items():
            self.category.choices[category_type.name] = list(categories)


def expenses_append_get():
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    # Currently we support adding expenses only to current year
    year_categories = requested_user.get_year_categories(datetime.now().year)

    form = AppendExpenseForm()
    form.populate_category_choices(year_categories)

    return render_template(
        "expenses_append.html",
        form=form,
        infos=flash_collect(),
        current_year=datetime.now().year,
    )


def _update_savings(requested_user: User, category: str, amount: float):
    savings = requested_user.get_savings()

    saving_record = savings.get_by_category().get(category, None)
    if saving_record is not None:
        amount += saving_record.balance

    savings.update(category, None, amount)
    savings.store()

    flash("Updated savings.", FlashType.INFO.name)


def expenses_append_post():
    requested_user: User | None = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    # Currently we support adding expenses only to current year
    year_expenses = requested_user.get_year_expenses(datetime.now().year)
    year_categories = requested_user.get_year_categories(datetime.now().year)

    form = AppendExpenseForm()
    form.populate_category_choices(year_categories)

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

        year_expenses.insert_expense(expense)

        if expense.category in year_categories[CategoryType.SAVINGS]:
            _update_savings(requested_user, expense.category, expense.amount)

    except FileNotFoundError:
        return redirect(
            url_for("main.expenses_create", year=form.expense_date.data.year)
        )

    flash("Request completed.", FlashType.INFO.name)
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
                "exception:": f"{e}",
            }
        ), 400

    # We support adding expences only to current year
    if expense.expense_date.year != datetime.now().date().year:
        return jsonify(
            {"status": f"Cannot add expense for year {expense.expense_date.year}."}
        ), 501

    try:
        year_categories = requested_user.get_year_categories(expense.expense_date.year)

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
        year_expenses = requested_user.get_year_expenses(expense.expense_date.year)
    except Exception:
        return jsonify(
            {
                "status": f"Could not load expenses for given year {expense.expense_date.year}"
            }
        ), 500

    try:
        year_expenses.insert_expense(expense)

        if expense.category in year_categories[CategoryType.SAVINGS]:
            _update_savings(requested_user, expense.category, expense.amount)
    except Exception:
        return jsonify(
            {
                "status": f"Could not append expense file for given year {expense.expense_date.year}."
            }
        ), 500

    return jsonify({"status": "Ok"}), 200
