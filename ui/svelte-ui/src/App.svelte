<script>
	import { SvelteToast, toast } from '@zerodevx/svelte-toast'
	import { AppWebsocket } from '@holochain/conductor-api';
import { bufferToBase64, encodeJson } from './utils';

// hc sandbox generate workdir/happ/ --run=8888 --app-id tragedy

 class AppClient {

    constructor(host, appPort) {
        this._host = host || 'localhost';
        this._appPort = appPort || '8888';
    }
    async connect() {
        this._appClient = await AppWebsocket.connect(
            `ws://${this._host}:${this._appPort}`,
            30000,
            (signal) => signalHandler(self, signal))

        this._appInfo = await this._appClient.appInfo({ installed_app_id: "tragedy_of_commons" });
        this._cellId = this._appInfo.cell_data[0].cell_id;
        this._agentPubKey = this._cellId[1]
        this._dna = this._cellId[0]
        this._dnaStr = bufferToBase64(this._dna)
        this.agentPubKeyB64 = bufferToBase64(this._agentPubKey);
        console.log("appinfo:{}", this._appInfo);
    }

    async close() {
        // same, client.close() doesn't work in the browser
        await this._appClient.client.socket.close();
    }

    async startNewGame(gameCode) {
        const params = {
            cap: null,
            cell_id: this._cellId,
            zome_name: 'tragedy_of_commons',
            fn_name: 'create_game_code_anchor',
            provenance: this._agentPubKey,
            payload: gameCode
        };
        return this._appClient.callZome(params);
    }

    async getPlayers(gameCode) {
        const params = {
            cap: null,
            cell_id: this._cellId,
            zome_name: 'tragedy_of_commons',
            fn_name: 'get_players_for_game_code',
            provenance: this._agentPubKey,
            payload: gameCode
        };
        return this._appClient.callZome(params);
    }

    async joinGame(gameCode, nickname) {
        console.log("joinGame:", gameCode, nickname);
        let payload = { gamecode: gameCode, nickname: nickname };
        const params = {
            cap: null,
            cell_id: this._cellId,
            zome_name: 'tragedy_of_commons',
            fn_name: 'join_game_with_code',
            provenance: this._agentPubKey,
            payload: payload,
        };
        return this._appClient.callZome(params);
    }

    async startGame(gameCode) {
        const params = {
            cap: null,
            cell_id: this._cellId,
            zome_name: 'tragedy_of_commons',
            fn_name: 'start_game_session_with_code',
            provenance: this._agentPubKey,
            payload: gameCode
        };
        return this._appClient.callZome(params);
    }
    async currentRoundForGameCode(gameCode) {
        try {
            const params = {
                cap: null,
                cell_id: this._cellId,
                zome_name: 'tragedy_of_commons',
                fn_name: 'current_round_for_game_code',
                provenance: this._agentPubKey,
                payload: gameCode
            };
            return this._appClient.callZome(params);
        } catch (error) {
            console.log('ERROR: callZome threw error', error)
            return error;
        }
    };

    // async currentRoundInfoForHeaderHash(entryHash) {
    //     const params = {
    //         cap: null,
    //         cell_id: this.#cellId,
    //         zome_name: 'tragedy_of_commons',
    //         fn_name: 'current_round_info',
    //         provenance: this.#agentPubKey,
    //         payload: entryHash
    //     };
    //     return this.#appClient.callZome(params);
    // }
    async makeMove(amount, prev_round_hash) {
            const params = {
                cap: null,
                cell_id: this._cellId,
                zome_name: 'tragedy_of_commons',
                fn_name: 'make_new_move',
                provenance: this._agentPubKey,
                payload: { resource_amount: parseInt(amount), previous_round: prev_round_hash }
            };
            return this._appClient.callZome(params);
        }
    async tryCloseRound(prev_round_hash) {
            const params = {
                cap: null,
                cell_id: this._cellId,
                zome_name: 'tragedy_of_commons',
                fn_name: 'try_to_close_round',
                provenance: this._agentPubKey,
                payload: prev_round_hash
            };
            return this._appClient.callZome(params);
        }


    async getMyOwnedSessions(amount, prev_round_hash) {
            const params = {
                cap: null,
                cell_id: this._cellId,
                zome_name: 'tragedy_of_commons',
                fn_name: 'get_my_owned_sessions',
                provenance: this._agentPubKey,
                payload: null
            };
            return this._appClient.callZome(params);
        }
    async getMyPlayedSessions(amount, prev_round_hash) {
            const params = {
                cap: null,
                cell_id: this._cellId,
                zome_name: 'tragedy_of_commons',
                fn_name: 'get_my_played_sessions',
                provenance: this._agentPubKey,
                payload: null
            };
            return this._appClient.callZome(params);
        }
    async getAllMySessions(amount, prev_round_hash) {
            const params = {
                cap: null,
                cell_id: this._cellId,
                zome_name: 'tragedy_of_commons',
                fn_name: 'get_all_my_sessions',
                provenance: this._agentPubKey,
                payload: null
            };
            return this._appClient.callZome(params);
        }
    async getMyActiveSessions(amount, prev_round_hash) {
            const params = {
                cap: null,
                cell_id: this._cellId,
                zome_name: 'tragedy_of_commons',
                fn_name: 'get_my_active_sessions',
                provenance: this._agentPubKey,
                payload: null
            };
            return this._appClient.callZome(params);
        }

    }

	/**********/
	import NavBar from "./NavBar.svelte";
	import StartMenu from "./StartMenu.svelte";
	import Game from "./Game.svelte";
	import { onDestroy, onMount } from "svelte";

	let queryParamPort = new URLSearchParams(window.location.search).get(
		"port"
	);

	let status = "START"; // "GAME_BEGIN"  "GAME_JOIN" "LOADING"
	let nickname = "---";
	let gamecode = "------";
	let errorMessage = "";
	let agentPubKey = "";

	async function startNewGame(event) {
		if (status === "LOADING") {
			// prevent multiple clicks
			return;
		}
		nickname = event.detail.nickname;
		gamecode = generateGameCode();
		try {
			status = "LOADING";
			const anchor = await window.appClient.startNewGame(gamecode);
			console.log("anchor", anchor);
			const result = await window.appClient.joinGame(gamecode, nickname);
			console.log("joined game", result);
			status = "GAME_BEGIN";
		} catch (error) {
			errorMessage = error.data?.data || error.message;
			console.log("error", error);
		}
	}

	async function joinGame(event) {
		if (status === "LOADING") {
			return;
		}
		status = "LOADING";
		nickname = event.detail.nickname;
		gamecode = event.detail.gamecode;
		console.log("nick and code: ", nickname, gamecode);
		console.log("gamecode", gamecode);
		const result = await window.appClient.joinGame(gamecode, nickname);
		console.log("joingame", result);
		status = "GAME_JOIN";
	}

	function generateGameCode() {
		return Math.random().toString(36).substr(2, 6).toUpperCase();
	}
	/****************************************/
	let appHost = "localhost";
	let appPort = queryParamPort || 8000;
	let appId = "tragedy_of_commons";
	let connected = false;
	let agentPubKeyB64 = "";

	async function connect() {
		if (connected && window.appClient) {
			window.appClient = null;
			connected = false;
			return;
		}
		const appClient = new AppClient(appHost, appPort);
		try {
			await appClient.connect();
			window.appClient = appClient;
			agentPubKeyB64 = appClient.agentPubKeyB64;
			connected = true;
			errorMessage = "";
		} catch (error) {
			errorMessage = error.data || error.message;
			connected = false;
		}
	}

	let game_ctrl;
	let current_round_hash;

	function signalHandler(connection, signal) {
		// ignore signals not meant for me
		// if (!connection.syn || bufferToBase64(signal.data.cellId[1]) != connection.syn.me) {
		//     return
		// }
		console.log('Got Signal', signal.data.payload.signal_name, signal)
		switch (signal.data.payload.signal_name) {
			case 'StartGame':
				toast.push('The game has started. \n Click Play!');
				current_round_hash = signal.data.payload.signal_payload.round_entry_hash_update;
				game_ctrl.startNextRound();
				break;
			case 'StartNextRound':
				toast.push('Next round!! \n Make your move...');
				game_ctrl.startNextRound();
				break
			case 'PlayerJoined':
				toast.push('Player '+ signal.data.payload.signal_payload.nickname + ' joined');
				// add to player list
				//game_ctrl.refreshPlayerList();
			default:
				toast.push('DEBUG:' + signal.data.payload.signal_name);
				console.log("signal data: {:?}", signal.data);
		}
	}

	onMount(async () => {
		console.log("on mount");
		connect();
	});

	onDestroy(() => {
		if (window.appClient) {
			window.appClient.close();
		}
	});
</script>

<NavBar />

{#if status === "START"}
	<StartMenu on:startNewGame={startNewGame} on:joinGame={joinGame} />
{:else if status == "LOADING"}
	<div style="flex-flow:column; text-align: center;">
		<h1>LOADING...</h1>
		<p>Connecting to conductor...</p>
		<p>Calling zome...</p>
	</div>
{:else}
	<Game id="game" action={status} {nickname} {gamecode} {agentPubKeyB64} {current_round_hash} bind:controller={game_ctrl}/>
{/if}

<footer>

	{#if errorMessage}
		<h3 style="color: red;">{errorMessage}</h3>
	{/if}
</footer>
<SvelteToast/>
<button on:click={() => toast.push('Hello world!')}>EMIT TOAST</button>

<style>
	h1 {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
	}
</style>
