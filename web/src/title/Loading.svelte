<script lang="ts">
  // Block the rest of the page while 'loading' is non-empty

  export let loading: string[];
</script>

{#if loading.length > 0}
  <div class="outer">
    <div class="inner">
      {#each loading as msg}
        <p>{msg}</p>
      {/each}
      <div class="scroll-to-bottom" />
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
    background: white;

    overflow: auto;
  }

  .inner * {
    overflow-anchor: none;
  }

  .scroll-to-bottom {
    overflow-anchor: auto;
    height: 1px;
  }
</style>
