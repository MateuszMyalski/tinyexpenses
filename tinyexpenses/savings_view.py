from flask import render_template, get_flashed_messages
from flask_login import current_user
from flask_wtf import FlaskForm
from wtforms import (
    HiddenField,
    SubmitField,
)
from .models.accounts import User
from .models.savings import Savings
from .models.flash import flash_collect
from .extensions import users_db
from collections import defaultdict


class SavingRecordForm(FlaskForm):
    category = HiddenField("Category")
    account = HiddenField("Account")
    balance = HiddenField("Balance")
    submit_edit = SubmitField("Edit")
    submit_withdraw = SubmitField("Withdraw")


def savings_view_get():
    requested_user: User | None = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    savings = requested_user.get_savings()

    savings_by_category_form = defaultdict(dict)

    for saving in savings.get_by_category().values():
        savings_by_category_form[saving.category] = SavingRecordForm()
        savings_by_category_form[saving.category].category.data = saving.category
        savings_by_category_form[saving.category].balance.data = str(saving.balance)
        savings_by_category_form[saving.category].account.data = saving.account

    return render_template(
        "savings_view.html",
        savings_by_account=savings.get_savings_by_account(),
        savings_by_category_form=savings_by_category_form,
        savings_account_totals=savings.get_savings_account_totals(),
        currency=requested_user.currency,
        infos=flash_collect(),
    )
