<script lang="ts">
  // TODO Upstream to svelte-utils

  // Block the rest of the page while 'loading' is non-empty

  export let loading: string[];

  let inner: HTMLDivElement;

  $: if (loading.length > 0 && inner) {
    inner.scrollTop = inner.scrollHeight;
  }
</script>

{#if loading.length > 0}
  <div class="outer">
    <div class="inner" bind:this={inner}>
      {#each loading as msg}
        <p>{msg}</p>
      {/each}
    </div>
  </div>
{/if}

<style>
  .outer {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 999;

    color: white;
    font-size: 32px;
  }

  .inner {
    width: 60%;
    height: 60%;
    background-color: var(--pico-background-color);

    overflow: auto;
  }
</style>
