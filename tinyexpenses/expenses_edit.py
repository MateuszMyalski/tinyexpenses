from flask import render_template, redirect, url_for
from flask_login import current_user
from .models.accounts import AppUser
from .extensions import users_db
from .models.expenses import YearExpensesReport, ExpenseRecord
from .csv_edit import render_csv_data_edit_form, handle_csv_data_edit


def _store_expenses_data_cb(ctx: dict, data: str):
    expenses = [ExpenseRecord(*row) for row in data]

    YearExpensesReport.store(ctx["db_file"], expenses)


def expenses_edit_post(year: int):
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    db_file = requested_user._get_year_expenses_file(year)

    if not db_file.exists():
        return redirect(url_for("main.expenses_create", year=year))
    
    return handle_csv_data_edit(
        url_for("main.expenses_edit", year=year),
        _store_expenses_data_cb,
        {"db_file": db_file},
    )


def expenses_edit_get(year: int):
    requested_user: AppUser | None = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    try:
        year_expenses = requested_user.get_year_expenses(year).get_expenses()
    except FileNotFoundError:
        return redirect(url_for("main.expenses_create", year=year))

    year_expenses.sort(key=lambda timestamp: timestamp.timestamp)

    return render_csv_data_edit_form(
        col_labels=[col.label for col in ExpenseRecord.Columns],
        csv_content=year_expenses,
    )
