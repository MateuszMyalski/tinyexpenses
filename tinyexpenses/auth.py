from tinyexpenses.models import User
from .django_http import url_has_allowed_host_and_scheme
from flask import abort, render_template, request, redirect, url_for
from flask_wtf import FlaskForm
from flask_login import login_user, logout_user
from wtforms import StringField, PasswordField, SubmitField, validators
from .extensions import login_manager, users_db

class LoginForm(FlaskForm):
    username = StringField('Username',  [validators.DataRequired()])
    password = PasswordField('Password',  [validators.DataRequired()])
    submit = SubmitField('Login')

@login_manager.user_loader
def auth_load_user(user_id):
    return users_db.get(user_id)

def auth_display_login_form():
    form = LoginForm()
    return render_template("login.html", form=form)

def auth_authenticate_user():
    form = LoginForm()
    user: User | None = users_db.get(form.username.data)

    if not form.validate():
        return render_template("login.html", form=form, message="Not validated")

    if user is None:
        return render_template("login.html", form=form)
    
    if not user.check_password(form.password.data):
        return render_template("login.html", form=form, message="Invalid username/password")
    
    login_user(user)

    next = request.args.get('next')
    # url_has_allowed_host_and_scheme should check if the url is safe
    # for redirects, meaning it matches the request host.
    # See Django's url_has_allowed_host_and_scheme for an example.
    if next and not url_has_allowed_host_and_scheme(next, request.host):
        return abort(400)

    return redirect(next or url_for('main.index'))

def auth_logout():
    logout_user()
    return redirect(url_for("main.index"))