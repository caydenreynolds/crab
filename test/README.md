# Crablang tests

These tests use the crabc compiler to ensure generated binaries perform as expected.
All tests use the [robot framework](https://robotframework.org/?tab=0#getting-started)

Once you've done the setup (below), running the tests is as easy as:
`./runtests.sh`

# Set up

* `python -m venv crab_env`
  * I'm using python 3.9
  * crab_env must be in the `test` directory for the scripts to run properly
* `source crab_env/Scripts/activate`
* `pip install robotframework`
* `./runtests.sh`
