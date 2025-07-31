from flask import render_template, redirect, url_for
from flask_login import current_user
from flask_wtf import FlaskForm
from wtforms import (
    SubmitField,
    FloatField,
    validators,
)
from .extensions import users_db


class SetInitialBalanceForm(FlaskForm):
    initial_balance_amount = FloatField(
        "Initial balance",
        [validators.DataRequired(message="Initial balance is required.")],
    )
    submit = SubmitField("Submit")


def create_new_year_report(year: int):
    form = SetInitialBalanceForm()

    if not form.validate_on_submit():
        return render_template(
            "expenses_create.html",
            form=form,
            year=year,
            infos=[("error", "Request could not be validated.")],
        )

    requested_user = users_db.get(current_user.id)
    if requested_user is None:
        return render_template("error.html", message="User not found.")

    requested_user.create_expenses_records(year, form.initial_balance_amount.data)

    return redirect(url_for("main.expenses_view_year", year=year))


def create_new_year_report_form(year: int):
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    expenses_file = requested_user.get_expenses_report_file(year)

    if expenses_file.exists():
        return redirect(url_for("main.expenses_view_year", year=year))

    form = SetInitialBalanceForm()
    return render_template("expenses_create.html", form=form, year=year)
