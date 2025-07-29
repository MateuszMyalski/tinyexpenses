from flask import render_template, redirect, url_for, flash, get_flashed_messages
from flask_login import current_user
from flask_wtf import FlaskForm
from wtforms import (
    SubmitField,
    DateField,
    FloatField,
    TextAreaField,
    SelectField,
    validators,
    ValidationError
)
from .models import Categories, YearExpensesReport, ExpenseRecord
from .extensions import users_db
from datetime import datetime, date



class AppendExpenseForm(FlaskForm):
    @staticmethod
    def validate_date(form, field):
        if field.data.year != date.today().year:
            raise ValidationError("Only dates from the current year are allowed.")

    category = SelectField(
        "Category", [validators.DataRequired()], choices=[], coerce=str
    )
    date = DateField(
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


def append_expense_form():
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    try:
        # Currently we support adding expenses only to current year
        categories = Categories(requested_user, datetime.now().year)
    except (FileNotFoundError, ValueError) as e:
        return render_template("error.html", message=str(e))

    info = get_flashed_messages(False, ["append_expense_info"])

    form = AppendExpenseForm()
    form.category.choices = categories.get_categories()
    return render_template("expenses_append.html", form=form, infos=info, current_year=datetime.now().year)


def append_expense():
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    try:
        # Currently we support adding expenses only to current year
        categories = Categories(requested_user, datetime.now().year)
    except Exception as e:
        return render_template("error.html", message=str(e))

    form = AppendExpenseForm()
    form.category.choices = categories.get_categories()

    if not form.validate_on_submit():
        return render_template(
            "expenses_append.html",
            form=form,
            infos=["Request could not be validated."],
            current_year=datetime.now().year,
        )

    try:
        expense = ExpenseRecord(
            timestamp=datetime.now().isoformat(),
            category=form.category.data,
            date=form.date.data,
            amount=form.amount.data,
            description=form.description.data,
        )

        YearExpensesReport.insert_expense(requested_user, expense)
    except FileNotFoundError as e:
        return redirect(url_for("main.expenses_create", year=form.date.data.year))
    except Exception as e:
        return render_template("error.html", message=str(e))

    flash("Request completed.", "append_expense_info")
    return redirect(url_for("main.expenses_append"))
