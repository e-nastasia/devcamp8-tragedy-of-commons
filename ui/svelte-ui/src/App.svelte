<script>
	import NavBar from "./NavBar.svelte";
	import StartMenu from "./StartMenu.svelte";
	import Game from "./Game.svelte";
	import { connection } from "./stores.js";
	import { Connection, Zome } from "./zome.js";

	const DELAY = 300;

	let status = "START"; // "GAME_BEGIN"  "GAME_JOIN" "LOADING"
	let nickname = "---";
	let gamecode = "------";

	function startNewGame(event) {
		if (status == "LOADING") {
			// prevent multiple clicks
			return;
		}
		status = "LOADING";
		nickname = event.detail.nickname;
		gamecode = generateGameCode();

		asyncCallZomeToStartNewGame();
	}

	function callZomeToStartNewGame() {
		return new Promise((resolve) => {
			setTimeout(() => {
				resolve("resolved");
			}, DELAY);
		});
	}

	async function asyncCallZomeToStartNewGame() {
		// call holochain conductor
		// wait for response
		// move to other screen
		const result = await callZomeToStartNewGame();
		status = "GAME_BEGIN";
	}

	function joinGame(event) {
		if (status == "LOADING") {
			return;
		}
		status = "LOADING";
		nickname = event.detail.nickname;
		gamecode = event.detail.gamecode;
		// call holochain conductor
		// wait for response
		// move to other screen
		asyncCallZomeToJoinGame(gamecode);
	}

	function callZomeToJoinGame(gamecode) {
		return new Promise((resolve) => {
			setTimeout(() => {
				resolve("resolved");
			}, DELAY);
		});
	}

	async function asyncCallZomeToJoinGame() {
		// call holochain conductor
		// wait for response
		// move to other screen
		const result = await callZomeToJoinGame();
		status = "GAME_JOIN";
	}

	function generateGameCode() {
		return Math.random().toString(36).substr(2, 6).toUpperCase();
	}
	/****************************************/

	let appHost = "localhost";
	let appPort = 8888;
	let appId = "tragedy";

	async function toggle() {
		if (!$connection) {
			$connection = new Connection(appHost, appPort, appId);
			await $connection.open();
			console.log("attaching...");
			let zome = new Zome($connection, appId);
			zome.attach();
			console.log("zome is attached:{}", zome.attached());
			// await $connection.joinSession();
			// sessions = $connection.sessions;
		} else {
			// $connection.syn.clearState();
			// sessions = undefined;
			console.log("TODO disconnected");
		}
	}
</script>

<NavBar />

{#if status == "START"}
	<StartMenu on:startNewGame={startNewGame} on:joinGame={joinGame} />
{:else if status == "LOADING"}
	<div style="flex-flow:column; text-align: center;">
		<h1>LOADING...</h1>
		<p>Connecting to conductor...</p>
		<p>Calling zome...</p>
	</div>
{:else}
	<Game action={status} {nickname} {gamecode} />
{/if}
<footer>
	<div
		style="display:flex; vertical-align:middle; justify-content:space-around;"
	>
		<div>
			<label>Host</label><input bind:value={appHost} />
		</div>
		<div>
			<label>Port</label>
			<input bind:value={appPort} />
		</div>
		<div>
			<label>AppId</label>
			<input bind:value={appId} />
		</div>
	</div>
		<div style="display:flex; justify-content:center;">
			<button class="linkbutton" on:click={toggle}>
				{#if $connection}
					disconnect
				{:else}
					connect
				{/if}
			</button>
		</div>
</footer>

<style>
	h1 {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
	}
	
</style>
