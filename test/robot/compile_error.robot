*** Settings ***
Documentation     A test suite for crabfiles that should always cause a compile error.
Resource          keywords/all.robot
Library           String

*** Keywords ***
The crab compiler exits with an error when the Crabfile "${crabfile}" is built"
    ${crabfile_name} =  Fetch From Left  ${crabfile}  .
    IF  "${VERBOSE}" == "TRUE"
        The Following Command Exits With An Error:  ${CRABC}  -c  ${CBUILTINS_DIR}  -o  ${TARGET_DIR}/${crabfile_name}.exe  --verify  -v  ${CRAB_SRC}/${crabfile}  ${CRAB_STD}
    ELSE
        The Following Command Exits With An Error:  ${CRABC}  -c  ${CBUILTINS_DIR}  -o  ${TARGET_DIR}/${crabfile_name}.exe  --verify  ${CRAB_SRC}/${crabfile}  ${CRAB_STD}
    END

*** Test Cases ***
Attempt to build invalid Crabfiles
    [Template]  The Crab Compiler Exits With An Error When The Crabfile "${crabfile}" Is Built"
    func_arg_type.crab
    func_no_arg.crab
    func_unexpected_arg.crab
    no_such_func.crab
    tmpl_wrong_type.crab
