from flask import render_template, redirect, url_for, flash, get_flashed_messages
from flask_login import current_user
from flask_wtf import FlaskForm
from wtforms import (
    HiddenField,
    SubmitField,
    StringField,
    validators,
    ValidationError,
)
from .models.accounts import User
from .models.savings import Savings
from .extensions import users_db
from .savings_view import SavingRecordForm
from .models.flash import FlashType, flash_collect

def non_negative(form, field):
    if field.data is not None and float(field.data) < 0:
        raise ValidationError("Value must not be negative.")


class SavingsEditForm(FlaskForm):
    category = HiddenField("Category")
    balance = StringField(
        "Amount",
        validators=[
            validators.DataRequired(),
            validators.InputRequired(),
            non_negative,
        ],
        description="Setting to 0 will remove the category",
    )
    account = StringField("Account", validators=[validators.DataRequired()])
    submit = SubmitField("Submit")


def _handle_view_form_post(saving_record_form: SavingRecordForm):
    requested_user: User | None = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    available_years = requested_user.get_available_expenses_reports()

    if len(available_years) <= 0:
        flash("No year expenses available.", FlashType.ERROR.name)
        return redirect(url_for("main.savings_view"))

    form = SavingsEditForm()
    form.balance.data = saving_record_form.balance.data
    form.balance.default = saving_record_form.balance.data
    form.account.data = saving_record_form.account.data
    form.account.default = saving_record_form.account.data
    form.category.data = saving_record_form.category.data

    return render_template(
        "savings_edit.html",
        category=saving_record_form.category.data,
        form=form,
    )


def _handle_withdraw_post():
    requested_user: User | None = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    form = SavingsEditForm()

    if not form.validate_on_submit():
        flash("Request could not be validated.", FlashType.ERROR.name)
        return redirect(url_for("main.savings_view"))

    savings_file = requested_user.get_savings_file()
    savings = Savings(savings_file)

    savings.update(form.category.data, form.account.data, form.balance.data)

    savings.store()

    flash(f"Edit of '{form.category.data}' succeed!", FlashType.INFO.name)

    return redirect(url_for("main.savings_view"))


def savings_edit_post():
    saving_record_form = SavingRecordForm()
    withdraw_form = SavingsEditForm()

    if saving_record_form.submit_edit.data:
        return _handle_view_form_post(saving_record_form)

    if withdraw_form.submit.data:
        return _handle_withdraw_post()

    return redirect(url_for("main.savings_view"))
