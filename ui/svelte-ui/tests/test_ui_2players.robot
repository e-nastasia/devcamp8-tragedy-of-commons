*** Settings ***
Documentation     Simple example using SeleniumLibrary.
Library           SeleniumLibrary

*** Variables ***
${URL_A}      http://localhost:5000?port=8000
${URL_B}      http://localhost:5000?port=8001
${BROWSER}        Firefox

*** Test Cases ***
Play Game
    Open Browser A To App Page
    Input Nickname    Tim
    Click Start
    ${GAMECODE}    Get Text    gamecode

    Open Browser B To App Page
    Input Nickname    Eva
    Input Gamecode    ${GAMECODE}
    Click Join

    Sleep    3s
    Switch Browser    A
    Refresh Player List

    Switch Browser    B
    Refresh Player List

    Switch Browser    A
    Click Play


   # [Teardown]    Close All Browsers

*** Keywords ***
Open Browser A To App Page
    Open Browser    ${URL_A}    ${BROWSER}  alias=A
    Title Should Be    Devcamp n°8 | september 2021

Open Browser B To App Page
    Open Browser    ${URL_B}    ${BROWSER}  alias=B
    Title Should Be    Devcamp n°8 | september 2021

Input Nickname
    [Arguments]    ${nickname}
    Input Text    input_start_nick    ${nickname}
    Input Text    input_join_nick     ${nickname}

Input Gamecode
    [Arguments]    ${gamecode}
    Input Text    input_join_game_code    ${gamecode}

Click Start
    Click Button    start_game_btn

Click Join
    Click Button    join_game_btn

Click Play
    Click Button    start_play_btn
Refresh Player List
    #tries max 3 times
    FOR    ${i}    IN RANGE    6
        Click Element   refresh_player_list
        ${player_count}=    Get Text    playercount
        ${has_2_players}=   Evaluate    ${playercount} == 2
        Exit For Loop If    ${has_2_players}
        Sleep    3s
        Log    try ${i}
    END