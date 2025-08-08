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
        [validators.InputRequired(message="Initial balance is required.")],
        default=0.0,
    )
    submit = SubmitField("Submit")


def expenses_create_post(year: int):
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

    requested_user.create_year_expenses(year, form.initial_balance_amount.data)

    return redirect(url_for("main.expenses_view_year", year=year))


def expenses_create_get(year: int):
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    try:
        requested_user.get_year_expenses(year)
    except FileNotFoundError:
        form = SetInitialBalanceForm()
        return render_template("expenses_create.html", form=form, year=year)
    
    return redirect(url_for("main.expenses_view_year", year=year))

