*** Settings ***
Documentation     Simple example using SeleniumLibrary.
Library           SeleniumLibrary

*** Variables ***
${URL_A}      http://localhost:5000?port=8000
${URL_B}      http://localhost:5000?port=8001
${BROWSER}        Firefox

*** Test Cases ***
Play Game
    # Open app as game host and start new game
    Open Browser A To App Page
    Input Nickname    Tim
    Click Start
    Sleep    1s
    ${GAMECODE}    Get Text    gamecode

    #Open app as participant and join game with same code
    Open Browser B To App Page
    Input Nickname    Eva
    Input Gamecode    ${GAMECODE}
    Click Join

    #Refresh playerlist with retry for game host
    Switch Browser    A
    Refresh Player List

    #Refresh playerlist with retry for game host
    Switch Browser    B
    Refresh Player List

    #Game host starts game by clicking Play
    Switch Browser    A
    Click Play
    Sleep    2s
    # Handle Alert      ACCEPT

    #Participant joins game by clicking Play
    Switch Browser    B
    Sleep    1s
    # Handle Alert      ACCEPT
    Click Play
    
    # ROUND ONE
    Switch Browser    A
    Take Resource    1
    
    Switch Browser    B
    Take Resource    2  
    
    Switch Browser    A
    Refresh Round 
    # Handle Alert      ACCEPT

    Switch Browser    B
    # Handle Alert      ACCEPT
    Refresh Round 

    # ROUND TWO
    Switch Browser    A
    Take Resource    3  

    Switch Browser    B
    Take Resource    4  
    Refresh Round 

    Switch Browser   A
    Refresh Round 

    # ROUND THREE
    Switch Browser    A
    Take Resource    5  

    Switch Browser    B
    Take Resource    6  
    Refresh Round 

    Switch Browser    A
    Refresh Round 

    #GAME OVER
    Refresh Scores

    Switch Browser    B
    Refresh Scores
    
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

Take Resource
    [Arguments]    ${amount}
    Input Text      input_take_resources    ${amount}
    Click Button    make_move_btn

Click Start
    Click Button    start_game_btn

Click Join
    Click Button    join_game_btn

Click Play
    Click Button    start_play_btn 



Refresh Scores
    FOR    ${i}    IN RANGE    6
        Click Link    refresh_scores_btn        
        Wait Until Element Is Visible    game_results_section    3s
        Log    try ${i}
    END
    
Refresh Player List
    #tries max 6 times
    FOR    ${i}    IN RANGE    6
        Click Element   refresh_player_list
        ${player_count}=    Get Text    playercount
        ${has_2_players}=   Evaluate    ${playercount} == 2
        Exit For Loop If    ${has_2_players}
        Sleep    3s
        Log    try ${i}
    END

Refresh Round
    Sleep    5s
    Click Button    refresh_round_btn
    # FOR    ${i}    IN RANGE    6
    #     Click Button    refresh_round_btn
    #     Wait Until Element Is Visible    input_take_resources    3s
    #     Log    try ${i}
    # END


# Refresh Scores
#     #tries max 3 times
#     FOR    ${i}    IN RANGE    6
#         Click Element   refresh_player_list
#         ${player_count}=    Get Text    playercount
#         ${has_2_players}=   Evaluate    ${playercount} == 2
#         Exit For Loop If    ${has_2_players}
#         Sleep    3s
#         Log    try ${i}
#     END