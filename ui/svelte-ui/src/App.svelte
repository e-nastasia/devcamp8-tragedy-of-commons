<script>
	import NavBar from "./NavBar.svelte";
	import StartMenu from "./StartMenu.svelte";
	import Game from "./Game.svelte";
	import { AppClient } from "./app-client";
	import { onMount } from 'svelte';

	const DELAY = 300;

	let status = "START"; // "GAME_BEGIN"  "GAME_JOIN" "LOADING"
	let nickname = "---";
	let gamecode = "------";
	let errorMessage = '';

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
			console.log('anchor', anchor);
			status = "GAME_BEGIN";
		} catch (error) {
			errorMessage = error.data?.data || error.message;
			console.log('error', error);
		}
	}

	async function asyncCallZomeToStartNewGame() {
		// call holochain conductor
		// wait for response
		// move to other screen
		const result = await callZomeToStartNewGame();
		status = "GAME_BEGIN";
	}

	function joinGame(event) {
		if (status === "LOADING") {
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

	onMount(async () => {
		const appClient = new AppClient();
		try {
			await appClient.connect();
			window.appClient = appClient;
		} catch (error) {
			errorMessage = error.data || error.message;
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
	<Game action={status} {nickname} {gamecode} />
{/if}
{#if errorMessage}
	<h3 style="color: red;">{errorMessage}</h3>	
{/if}

<style>
	h1 {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
	}
	
</style>
