module.exports = {
    apps: [
        {
            name: "inkasso",
            script: "app.py",
            interpreter: "venv/bin/python3",
            interpreter_args: "-u",
            env: {
                FLASK_ENV: "production",
                FLASK_APP: "app.py"
            }
        }
    ]
};
