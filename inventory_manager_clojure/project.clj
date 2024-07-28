(defproject inventory_manager_clojure "0.1.0-SNAPSHOT"
  :description "FIXME: write description"
  :url "http://example.com/FIXME"
  :license {:name "EPL-2.0 OR GPL-2.0-or-later WITH Classpath-exception-2.0"
            :url "https://www.eclipse.org/legal/epl-2.0/"}
  :dependencies [[org.clojure/clojure "1.11.3"]
                 [org.clojure/core.async "1.3.610"]]
  :plugins [[lein-cljfmt "0.7.0"]]
  :cljfmt {:indents {defn [[:inner 0]]
                     defmethod [[:inner 0]]
                     defprotocol [[:inner 0]]}
           :remove-consecutive-blank-lines? true
           :remove-trailing-whitespace? true}
  :main inventory_manager_clojure.inventory-manager
  :target-path "target/%s"
  :profiles {:uberjar {:aot :all
                       :jvm-opts ["-Dclojure.compiler.direct-linking=true"]}})
