from .models.accounts import AppUser
from .django_http import url_has_allowed_host_and_scheme
from flask import abort, render_template, request, redirect, url_for
from flask_wtf import FlaskForm
from flask_login import login_user, logout_user
from wtforms import StringField, PasswordField, SubmitField, BooleanField, validators
from .extensions import login_manager, users_db


class LoginForm(FlaskForm):
    username = StringField("Username", [validators.DataRequired()])
    password = PasswordField("Password", [validators.DataRequired()])
    remember = BooleanField('Remember Me')
    submit = SubmitField("Login")


@login_manager.user_loader
def auth_load_user(user_id):
    return users_db.get(user_id)


def auth_display_login_form():
    form = LoginForm()
    return render_template("login.html", form=form)


def auth_authenticate_post():
    form = LoginForm()
    user: AppUser | None = users_db.get(form.username.data)

    if not form.validate_on_submit():
        return render_template("login.html", form=form, infos=[("error","Not validated")])

    if user is None:
        return render_template("login.html", form=form)

    if not user.check_password(form.password.data):
        return render_template(
            "login.html", form=form, infos=[("error", "Invalid username/password")]
        )

    if not login_user(user, remember=form.remember.data):
        return render_template(
            "login.html", form=form, infos=[("error", "User inactive")]
        )

    next = request.args.get("next")
    # url_has_allowed_host_and_scheme should check if the url is safe
    # for redirects, meaning it matches the request host.
    # See Django's url_has_allowed_host_and_scheme for an example.
    if next and not url_has_allowed_host_and_scheme(next, request.host):
        return abort(400)

    return redirect(next or url_for("main.index"))


def auth_logout():
    logout_user()
    return redirect(url_for("main.index"))
