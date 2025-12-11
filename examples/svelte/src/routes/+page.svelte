<script lang='ts'>
  import { invoke } from '@tauri-apps/api/core';

  let name = $state('');
  let greetMsg = $state('');

  async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke('greet', { name });
  }
</script>

<main class='flex flex-col min-h-screen w-full p-5 items-center'>

  <div class='flex flex-col gap-3 items-center justify-center'>
    <h1>Welcome to Tauri + Svelte</h1>
    <h2>Tauri Plugin Rstate Demo App</h2>
  </div>

  <div class='flex items-center gap-7 justify-center'>
    <a href='https://vite.dev' target='_blank'>
      <img src='/vite.svg' class='logo vite' alt='Vite Logo' />
    </a>
    <a href='https://tauri.app' target='_blank'>
      <img src='/tauri.svg' class='logo tauri' alt='Tauri Logo' />
    </a>
    <a href='https://svelte.dev' target='_blank'>
      <img src='/svelte.svg' class='logo svelte-kit' alt='SvelteKit Logo' />
    </a>
  </div>
  <p>Click on the Tauri, Vite, and SvelteKit logos to learn more.</p>

  <form class='row' onsubmit={greet}>
    <input id='greet-input' placeholder='Enter a name...' bind:value={name} />
    <button type='submit'>Greet</button>
  </form>
  <p>{greetMsg}</p>
</main>

<style>
.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.svelte-kit:hover {
  filter: drop-shadow(0 0 2em #ff3e00);
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}

</style>
