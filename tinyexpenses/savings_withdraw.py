from flask import render_template, redirect, url_for, flash
from flask_login import current_user
from flask_wtf import FlaskForm
from wtforms import (
    HiddenField,
    SubmitField,
    FloatField,
    SelectField,
    validators,
    ValidationError,
)
from .models.accounts import User
from .models.savings import Savings
from .models.expenses import YearExpensesReport, ExpenseRecord
from .models.categories import YearCategories, CategoryRecord, CategoryType
from .extensions import users_db
from .savings_view import SavingRecordForm
from .models.flash import FlashType, flash_collect

from datetime import datetime


def non_negative(form, field):
    if field.data is not None and field.data < 0:
        raise ValidationError("Value must not be negative.")


class SavingsWithdrawForm(FlaskForm):
    category = HiddenField("Category", validators=[validators.DataRequired()])
    balance = HiddenField("Balance", validators=[validators.DataRequired()])
    amount = FloatField(
        "Amount to withdraw",
        validators=[
            validators.DataRequired(),
            validators.InputRequired(),
            non_negative,
        ],
    )
    year_select = SelectField(
        "Withdraw to year", validators=[validators.DataRequired()], choices=[]
    )
    submit = SubmitField("Submit")

    def populate_year_choices(self, years: list[int]):
        years.sort(reverse=True)

        self.year_select.choices = list((str(year), str(year)) for year in years)


def _handle_view_form_post(saving_record_form: SavingRecordForm):
    requested_user: User | None = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    available_years = requested_user.get_available_expenses_reports()

    if len(available_years) <= 0:
        flash("No year expenses available.", FlashType.ERROR.name)
        return redirect(url_for("main.savings_view"))

    form = SavingsWithdrawForm()
    form.populate_year_choices(available_years)
    form.category.data = saving_record_form.category.data
    form.balance.data = saving_record_form.balance.data
    form.year_select.data = str(form.year_select.choices[-1])

    return render_template(
        "savings_withdraw.html",
        category=saving_record_form.category.data,
        balance=saving_record_form.balance.data,
        currency=requested_user.currency,
        form=form,
    )


def _handle_withdraw_post():
    requested_user: User | None = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")
    available_years = requested_user.get_available_expenses_reports()

    if len(available_years) <= 0:
        flash("No year expenses available.", FlashType.ERROR.name)
        return redirect(url_for("main.savings_view"))

    form = SavingsWithdrawForm()
    form.populate_year_choices(available_years)

    if not form.validate_on_submit():
        flash("Request could not be validated.", FlashType.ERROR.name)
        return redirect(url_for("main.savings_view"))

    savings_file = requested_user.get_savings_file()
    savings = Savings(savings_file)

    year_expenses_file = requested_user.get_expenses_report_file(form.year_select.data)

    withdrawed_amount = float(form.amount.data)

    saving_transfer = ExpenseRecord(
        timestamp=datetime.now().isoformat(),
        category=form.category.data,
        expense_date=datetime.now().date(),
        amount=-withdrawed_amount,
        description=f"Transfer of savings from {form.category.data}",
    )

    saving_record = savings.get_by_category()[form.category.data]
    saving_record.balance -= withdrawed_amount

    savings.update(form.category.data, None, saving_record.balance)
    savings.store()

    YearExpensesReport.insert_expense(year_expenses_file, saving_transfer)

    year_categories_file = requested_user.get_categories_file(form.year_select.data)
    year_categories = YearCategories(year_categories_file)

    new_saving_category = CategoryRecord(form.category.data, CategoryType.SAVINGS.name)

    year_categories.insert_category(new_saving_category)

    flash(f"Withdraw from '{form.category.data}' succeed!", FlashType.INFO.name)

    return redirect(url_for("main.savings_view"))


def savings_withdraw_post():
    saving_record_form = SavingRecordForm()
    withdraw_form = SavingsWithdrawForm()

    if saving_record_form.submit_withdraw.data:
        return _handle_view_form_post(saving_record_form)

    if withdraw_form.submit.data:
        return _handle_withdraw_post()

    return redirect(url_for("main.savings_view"))
