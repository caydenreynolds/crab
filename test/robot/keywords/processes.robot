*** Settings ***
Documentation     Resources for running processes.
...               Results may be checked immediately and/or later
...               All stdout and stderr is forwarded to the log file
Library           Process
Library           OperatingSystem


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

The last process output "${output}" matches the file "${expected_file}"
    ${file_contents} =  Get File  ${RESOURCES}/${expected_file}
    IF  "${output}" == "stdout"
        ${output_contents} =  Set Variable  ${last_process_result.stdout}
    ELSE IF  "${output}" == "stderr"
        ${output_contents} =  Set Variable  ${last_process_result.stderr}
    END
    Should be Equal As Strings  ${file_contents}  ${last_process_result.stdout}
