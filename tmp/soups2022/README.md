# Datasets
This directory contains the raw data and analysis scripts used in the paper "Improving Password Generation Through the Design of a Password Composition Policy Description Language" by Gautam et al.

## Directories
* **pcp dataset**—Contains the raw data and analysis for the PCP dataset reported on in the paper.
* **user study**—Contains the raw data for the user study reported on in the paper. This data includes some analysis as well.

## Running the analysis code
Analysis is done using iPython notebooks. We have used `pyenv` and `pipenv` to ensure a consistent runtime environment. To setup the project, you will need to have both of these tools installed. You will then run the following commands:
1. `pipenv install`
2. `pipenv run jupyter-notebook`

You can then open and execute the analysis scripts.