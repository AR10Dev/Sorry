import { Component, createSignal } from 'solid-js'

const App: Component = () => {
  const [count, setCount] = createSignal(0)

  return (
    <button
      type="button"
      class="text-white bg-gradient-to-r from-purple-500 via-purple-600 to-purple-700 hover:bg-gradient-to-br focus:ring-4 focus:ring-purple-300 dark:focus:ring-purple-800 shadow-lg shadow-purple-500/50 dark:shadow-lg dark:shadow-purple-800/80 font-medium rounded-2xl h-20 px-5"
      onClick={() => setCount(count() + 1)}
    >
      <p class="text-4xl font-mono text-gray-300 text-center select-none">
        You said "sorry" {count()} times
      </p>
    </button>
  )
}

export default App
