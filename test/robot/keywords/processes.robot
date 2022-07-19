*** Settings ***
Documentation     Resources for running processes.
...               Results may be checked immediately and/or later
...               All stdout and stderr is forwarded to the log file
Library           Process


*** Keywords ***
The following command is run:
    [Arguments]    @{varargs}  ${return_code}=0

    Run Keyword If  "${VERBOSE}" == "TRUE"  Log to Console  Running processs @{varargs}
    ${result} =  Run Process  @{varargs}  shell=True
    Log Many  stdOut:  ${result.stdout}
    Log Many  stdErr:  ${result.stderr}
    Set Test Variable  $last_process_result  ${result}

    Should Be Equal As Integers  ${return_code}  ${result.rc}

The following command exits with an error:
    [Arguments]    @{varargs}

    Run Keyword If  "${VERBOSE}" == "TRUE"  Log to Console  Running processs @{varargs}
    ${result} =  Run Process  @{varargs}  shell=True
    Log Many  stdOut:  ${result.stdout}
    Log Many  stdErr:  ${result.stderr}
    Set Test Variable  $last_process_result  ${result}

    Should Not Be Equal As Integers  ${result.rc}  0

The last process printed "${output}"
    Should be Equal As Strings  ${output}  ${last_process_result.stdout}
