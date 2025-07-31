from flask import render_template, redirect, url_for
from flask_login import current_user
from .models.accounts import User
from .extensions import users_db
from .models.expenses import YearExpensesReport, ExpenseRecord
from .csv_edit import render_csv_data_edit_form, handle_csv_data_edit


def _store_expenses_data_cb(ctx: dict, data: str):
    expenses = [ExpenseRecord(*row) for row in data]

    YearExpensesReport.store(ctx["db_file"], expenses)


def edit_expenses(year: int):
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    year_expenses_file = requested_user.get_expenses_report_file(year)

    if not year_expenses_file.exists():
        return redirect(url_for("main.expenses_create", year=year))

    return handle_csv_data_edit(
        url_for("main.expenses_edit", year=year),
        _store_expenses_data_cb,
        {"db_file": year_expenses_file},
    )


def edit_expenses_form(year: int):
    requested_user: User | None = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    year_expenses_file = requested_user.get_expenses_report_file(year)

    if not year_expenses_file.exists():
        return redirect(url_for("main.expenses_create", year=year))

    year_expenses: list[ExpenseRecord] = YearExpensesReport(
        year_expenses_file
    ).get_expenses()

    return render_csv_data_edit_form(
        [col.label for col in ExpenseRecord.Columns], year_expenses
    )
