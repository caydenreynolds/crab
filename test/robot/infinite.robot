*** Settings ***
Documentation     A test suite for simple crabfiles that will never exit on their own
Resource          keywords/all.robot
Library           Process
Library           String

*** Test Cases ***
Run loop test
    The Crabfile "inf_loop.crab" is built
    ${process} =  The "inf_loop" Crab application is started
    Sleep  5s  Give the loop some time to do its thing
    Process Should be Running  ${process}
    ${result} =  Terminate Process  ${process}
    ${total_line_count} =  Get Line Count  ${result.stdout}
    ${lines_matching_expected} =  Get Lines Matching Pattern  ${result.stdout}  We're going to be here for a while
    ${matching_line_count} =  Get Line Count  ${lines_matching_expected}
    Should Be Equal  ${total_line_count}  ${matching_line_count}
