from flask import render_template, redirect, url_for
from flask_login import current_user
from flask_wtf import FlaskForm
from wtforms import (
    SubmitField,
    SelectField,
    validators,
)
from .models import Categories, YearExpensesReport
from .extensions import users_db
from datetime import datetime


class CreateCategoryFileForm(FlaskForm):
    template_year = SelectField(
        "Category file template", [validators.DataRequired()], choices=[]
    )
    submit = SubmitField("Create categories")


def create_new_categories_form(year: int):
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    try:
        requested_user.get_categories_file(int(year))
    except FileNotFoundError:
        form = CreateCategoryFileForm()

        available_years = [
            *requested_user.get_available_categories_files(),
            int(year),
        ]
        available_years.sort(reverse=True)

        form.template_year.choices = [
            (str(available_year), str(available_year))
            for available_year in available_years
        ]

        form.template_year.label.text += f" (empty {year})"

        form.template_year.data = str(year)

        return render_template("categories_create.html", form=form, year=year)
    except Exception as e:
        return render_template("error.html", message=str(e))

    return redirect(url_for("main.categories_create", year=year))


def create_new_categories(year: int):
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    form = CreateCategoryFileForm()
    available_years = [
        *requested_user.get_available_categories_files(),
        int(year),
    ]
    available_years.sort(reverse=True)

    form.template_year.choices = [
        (str(available_year), str(available_year)) for available_year in available_years
    ]

    if not form.validate_on_submit():
        return render_template(
            "categories_create.html",
            form=form,
            year=year,
            infos=["Request could not be validated."],
        )

    try:
        if form.template_year.data != year:
            requested_user.create_categories_file(year, form.template_year.data)
        else:
            requested_user.create_categories_file(year)
    except Exception as e:
        return render_template("error.html", message=str(e))

    return redirect(url_for("main.expenses_view_year", year=year))
