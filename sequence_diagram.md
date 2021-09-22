You can preview in vscode by installing 
a plugin: vstirbu.vscode-mermaid-preview

Start of game:

```mermaid
sequenceDiagram
    participant A as Alice UI
    participant AZ as Alice zome
    participant DHT
    participant BZ as Bob zome
    participant B as Bob UI

    %% invitation fase
    A    ->>   AZ: create_game_code_anchor
    AZ  -)   DHT:         create anchor
    Note right of DHT: not of much <br>use if we don't <br>use hash
    
    A    ->>   AZ: join_game_with_code
    AZ  -)   DHT:         create anchor
    AZ  -)   DHT:         create profile
    AZ  -)   DHT:         link from anchor to profile<br>with tag PLAYER

    B    ->>   BZ: join_game_with_code
    BZ  -)   DHT:         create anchor
    BZ  -)   DHT:         create profile
    BZ  -)   DHT:         link from anchor to profile<br>with tag PLAYER

    %% check if all players are present
    A   ->> AZ:     get_players_for_game_code
    AZ  ->> DHT:    get links from anchor to players profiles
    B   ->> BZ:     get_players_for_game_code
    BZ  ->> DHT:    get links from anchor to players profiles

    %% player A starts game (by clicking Play)
    A   ->> AZ:     start_game_session_with_code  
    AZ  -) DHT:     get all players
    AZ  -) DHT:     create game session
    AZ  -) DHT:     create link from anchor to game session with tag GAME_SESSION
    AZ  -) DHT:     create round zero
    AZ  -) DHT:     create link from game session to game round with tag GAME_ROUND

    %% player B polls for game round (by clicking Play)
    B  ->> BZ:      current_round_for_game_code
    BZ -)  DHT:     get anchor
    BZ -)  DHT:     get link from anchor with tag GAME_SESSION
    BZ -)  DHT:     get link from game session with tag GAME_ROUND
    BZ -)  DHT:     get latest version of game round
    BZ  -->> B:     entry hash of game round

    %% player A makes move
    A ->> AZ:       make_new_move
    AZ ->> DHT:     create game move with reference to game round (for uniqueness)
    AZ ->> DHT:     create link from game round to game move
    AZ ->> A:       header hash from move
    A  ->> AZ:      try_to_close_round
    AZ ->> A:       return game info with next action

    %% player B makes move
    B ->> BZ:       make_new_move
    BZ ->> DHT:     create game move with reference to game round (for uniqueness)
    BZ ->> DHT:     create link from game round to game move
    BZ ->> B:       header hash from move
    A  ->> AZ:      try_to_close_round
    AZ ->> A:       return game info with next action


    alt play next round
    A ->> AZ:       make new move ...
    B ->> BZ:       make new move ...
    end

    A  ->> AZ:      try_to_close_round
    AZ ->> A:       return game info with next action: GAME OVER
    B  ->> BZ:      try_to_close_round
    BZ ->> B:       return game info with next action: GAME OVER

```

Starting next round

```mermaid
sequenceDiagram
    participant A as Alice UI
    participant AZ as Alice zome
    participant DHT
    participant BZ as Bob zome
    participant B as Bob UI

    B      ->>  BZ:    close round? yes
    BZ    ->>   BZ:    calculate result<br>round complete? yes<br>game complete? no
    BZ    ->>   DHT:         create round one
    BZ    -)    AZ:  remote signal make move
    BZ    ->>   B:      signal make move
    AZ  ->>   A:    signal make move

```

Ending game

```mermaid
sequenceDiagram
    participant A as Alice UI
    participant AZ as Alice zome
    participant DHT
    participant BZ as Bob zome
    participant B as Bob UI

    B      ->>  BZ:    close round? yes
    BZ    ->>   BZ:    calculate result<br>round complete? yes<br>game complete? yes
    BZ    ->>   DHT:         create round x<br>(no signal to make moves)
    BZ    ->>   DHT:         update game session
    BZ    -)    AZ:  remote signal game ended
    BZ    ->>   B:      signal game over
    AZ  ->>   A:    signal game over

```