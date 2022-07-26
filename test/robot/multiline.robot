*** Settings ***
Documentation     A test suite for slightly more complex crabfiles that take no args, always return 0, and print some output on any number of lines.
Resource          keywords/all.robot
Library           String
Library           paths

*** Keywords ***
The Crabfile "${crabfile}" is built and the results are compared against a file"
        The Crabfile "${crabfile}" is built
        ${crabfile_name} =  Fetch From Left  ${crabfile}  .
        The "${crabfile_name}" Crab application is run successfully
        ${at_least_one_checked} =  ${FALSE}
        FOR  ${output}  IN  stdout  stderr
            IF  File Exists  "${RESOURCSE}/multiline/${crabfile_name}.${output}"
                The last process output "${output}" matches the file "multiline/${crabfile_name}.${output}"
                ${at_least_one_checked} =  ${TRUE}
            END
        END
        Should Be True  ${at_least_one_checked}

*** Test Cases ***
Run Simple Crabfiles
    [Template]  The Crabfile "${crabfile}" is built and the results are compared against a file"
    struct_tmpl.crab
