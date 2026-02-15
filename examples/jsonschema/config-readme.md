## `Cfg`

<table>
<thead><tr><th>Field</th><th>Type</th><th>Default</th><th>Description</th></tr></thead>
<tbody>
<tr>
<td><code>connectionTimeout</code></td>
<td><a href="#durationstring"><code>DurationString</code></a></td>
<td><code class="language-json">&quot;15s&quot;</code></td>
<td>Connection timeout</td>
</tr>
<tr>
<td><code>hosts</code></td>
<td><code>array</code></td>
<td><code class="language-json">[]</code></td>
<td>List of hosts to index</td>
</tr>
<tr>
<td><code>mode</code></td>
<td><a href="#mode"><code>Mode</code></a></td>
<td><pre><code class="language-json">{
  &quot;kind&quot;: &quot;Serial&quot;
}</code></pre></td>
<td>Indexing mode</td>
</tr>
</tbody>
</table>

## `DurationString`

Base type: `string`

## `Mode`

<table>
<thead><tr><th>Variants</th></tr></thead>
<tbody>
<tr><td><a href="#modeserial"><code>Serial</code></a></td></tr>
<tr><td><a href="#modeparallel"><code>Parallel</code></a></td></tr>
</tbody>
</table>


## `Mode::Serial`

Index in a single thread

<table>
<thead><tr><th>Field</th><th>Type</th><th>Default</th><th>Description</th></tr></thead>
<tbody>
<tr>
<td><code>kind</code></td>
<td><code>string</code></td>
<td></td>
<td></td>
</tr>
</tbody>
</table>


## `Mode::Parallel`

Index in multiple threads

<table>
<thead><tr><th>Field</th><th>Type</th><th>Default</th><th>Description</th></tr></thead>
<tbody>
<tr>
<td><code>concurrency</code></td>
<td><code>integer</code></td>
<td><code class="language-json">10</code></td>
<td>Maximum concurrent requests</td>
</tr>
<tr>
<td><code>kind</code></td>
<td><code>string</code></td>
<td></td>
<td></td>
</tr>
</tbody>
</table>
