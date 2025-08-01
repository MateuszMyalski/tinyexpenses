from flask import render_template, redirect, url_for
from flask_login import current_user
from flask_wtf import FlaskForm
from wtforms import (
    SubmitField,
    SelectField,
    validators,
)
from .extensions import users_db


class CreateCategoryFileForm(FlaskForm):
    template_year = SelectField(
        "Category file template", [validators.DataRequired()], choices=[]
    )
    submit = SubmitField("Create")

    def populate_template_year_choices(self, years: list[int]):
        self.template_year.choices = list((str(year), str(year)) for year in years)


def categories_create_get(year: int):
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    categories_file = requested_user.get_categories_file(year)

    if categories_file.exists():
        return redirect(url_for("main.categories_create", year=year))

    available_years = [
        *requested_user.get_available_categories_files(),
        int(year),
    ]
    available_years.sort(reverse=True)

    form = CreateCategoryFileForm()
    form.populate_template_year_choices(available_years)
    form.template_year.label.text += f" (empty {year})"
    form.template_year.data = str(year)

    return render_template("categories_create.html", form=form, year=year)


def categories_create_post(year: int):
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    available_years = [
        *requested_user.get_available_categories_files(),
        int(year),
    ]
    available_years.sort(reverse=True)

    form = CreateCategoryFileForm()
    form.populate_template_year_choices(available_years)

    if not form.validate_on_submit():
        return render_template(
            "categories_create.html",
            form=form,
            year=year,
            infos=[("error", "Request could not be validated.")],
        )

    if form.template_year.data != year:
        requested_user.create_categories_file(year, form.template_year.data)
    else:
        requested_user.create_categories_file(year)

    return redirect(url_for("main.expenses_view_year", year=year))
