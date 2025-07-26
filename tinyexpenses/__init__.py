from flask import Flask
from .extensions import login_manager, users_db, format_number
from .routes import bp


def create_app():
    app = Flask(__name__)

    # TODO change to real secret
    app.secret_key = "secret"
    app.jinja_env.filters['format_number'] = format_number

    login_manager.init_app(app)
    
    # TODO: change it to use config file and proper path
    users_db.load("accounts")

    app.register_blueprint(bp)

    return app