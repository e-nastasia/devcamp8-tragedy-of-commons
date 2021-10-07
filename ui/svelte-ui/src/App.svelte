<script>
	import NavBar from "./NavBar.svelte";
	import StartMenu from "./StartMenu.svelte";
	import Game from "./Game.svelte";
	import { AppClient } from "./app-client";
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
	<Game action={status} {nickname} {gamecode} {agentPubKeyB64}/>
{/if}

<footer>

	{#if errorMessage}
		<h3 style="color: red;">{errorMessage}</h3>
	{/if}
</footer>

<style>
	h1 {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
	}
</style>
