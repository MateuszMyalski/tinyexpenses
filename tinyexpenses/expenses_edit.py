from flask import render_template, redirect, url_for
from flask_login import current_user
from flask_wtf import FlaskForm
from wtforms import (
    SubmitField,
    HiddenField,
)

from tinyexpenses.models import User
from .extensions import users_db
from .models import YearExpensesReport, ExpenseRecord
import json

class FileEditForm(FlaskForm):
    table_data = HiddenField("Table Data")
    submit = SubmitField("Save")


def edit_expenses(year: int):
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    form = FileEditForm()

    if not form.validate_on_submit():
        return render_template(
            "expenses_create.html", form=form, year=year, infos=["Request could not be validated."]
        )


    modified_expenses = []

    try:
        table_data = json.loads(form.table_data.data)
        for row in table_data:
            modified_expenses.append(ExpenseRecord(*row))
        
        YearExpensesReport.store_expenses(requested_user, int(year), modified_expenses)

    except Exception as e:
        return render_template("error.html", message=str(e))

    return redirect(url_for("main.expenses_edit", year=year))


def edit_expenses_form(year: int):
    requested_user: User | None = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    try:
        expenses = YearExpensesReport(requested_user, int(year)).get_expenses()
    except FileNotFoundError:
        return redirect(url_for("main.expenses_create", year=year))
    except Exception as e:
        return render_template("error.html", message=str(e))

    form = FileEditForm()

    return render_template("expenses_edit.html", form=form, expenses=expenses)
