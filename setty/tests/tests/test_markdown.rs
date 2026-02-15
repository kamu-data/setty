#![cfg(feature = "derive-jsonschema")]

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_markdown() {
    #[derive(setty::Config)]
    struct Cfg {
        /// Required
        required: String,

        /// With default
        #[config(default = "foo")]
        with_default: String,

        /// Option
        option: Option<String>,

        /// Nested
        #[config(default)]
        nested: Foo,

        /// Multi-line
        ///
        /// Description.
        ///
        /// Has many lines.
        multiline_desc: u32,

        /// Lorem ipsum
        ///
        /// ```sh
        /// cat "foo"
        /// ```
        code_desc: u32,
    }

    #[derive(setty::Config, setty::Default)]
    struct Foo {
        #[config(default)]
        a: u32,
    }

    pretty_assertions::assert_eq!(
        setty::Config::<Cfg>::new().markdown(),
        indoc::indoc!(
            r###"
            ## `Cfg`

            <table>
            <thead><tr><th>Field</th><th>Type</th><th>Default</th><th>Description</th></tr></thead>
            <tbody>
            <tr>
            <td><code>code_desc</code></td>
            <td><code>integer</code></td>
            <td></td>
            <td>

            Lorem ipsum

            ```sh
            cat "foo"
            ```

            </td>
            </tr>
            <tr>
            <td><code>multiline_desc</code></td>
            <td><code>integer</code></td>
            <td></td>
            <td>

            Multi-line

            Description.

            Has many lines.

            </td>
            </tr>
            <tr>
            <td><code>nested</code></td>
            <td><a href="#foo"><code>Foo</code></a></td>
            <td><pre><code class="language-json">{
              &quot;a&quot;: 0
            }</code></pre></td>
            <td>Nested</td>
            </tr>
            <tr>
            <td><code>option</code></td>
            <td><code>string</code></td>
            <td><code class="language-json">null</code></td>
            <td>Option</td>
            </tr>
            <tr>
            <td><code>required</code></td>
            <td><code>string</code></td>
            <td></td>
            <td>Required</td>
            </tr>
            <tr>
            <td><code>with_default</code></td>
            <td><code>string</code></td>
            <td><code class="language-json">&quot;foo&quot;</code></td>
            <td>With default</td>
            </tr>
            </tbody>
            </table>

            ## `Foo`

            <table>
            <thead><tr><th>Field</th><th>Type</th><th>Default</th><th>Description</th></tr></thead>
            <tbody>
            <tr>
            <td><code>a</code></td>
            <td><code>integer</code></td>
            <td><code class="language-json">0</code></td>
            <td></td>
            </tr>
            </tbody>
            </table>
            "###
        )
    );
}

/////////////////////////////////////////////////////////////////////////////////////////
