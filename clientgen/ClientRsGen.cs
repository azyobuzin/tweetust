using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using RestApisGen;

namespace Tweetust.ClientGen
{
    class ClientRsGen
    {
        public ClientRsGen(IEnumerable<ApiParent> apis, TextWriter writer)
        {
            this.apis = apis.Select(api => new Tuple<string, IReadOnlyList<RsEndpoint>>(
                api.Name,
                api.Endpoints.Where(x => !(x is RawLines)).Select(x => new RsEndpoint(x)).ToArray()
            )).ToArray();
            this.writer = writer;
        }

        private readonly IReadOnlyList<Tuple<string, IReadOnlyList<RsEndpoint>>> apis;
        private readonly TextWriter writer;

        public void Generate()
        {
            Header();

            foreach (var api in apis)
                ClientFactory(api.Item1);

            writer.WriteLine('}');

            foreach (var api in apis)
            {
                ClientStruct(api.Item1);
                ClientImpl(api.Item1, api.Item2);
            }
            
            foreach (var api in apis)
            {
                foreach (var endpoint in api.Item2)
                {
                    if (!CheckUnsupportedType(endpoint)) continue;
                    var name = string.Concat(api.Item1, endpoint.Name, "RequestBuilder");
                    RequestBuilderStruct(name, endpoint);
                    RequestBuilderImpl(name, endpoint);
                }
            }
        }

        private static string ToSnakeCase(string s)
        {
            if (s == "") return "";
            var sb = new StringBuilder(s.Length * 2);
            sb.Append(char.ToLowerInvariant(s[0]));
            for (var i = 1; i < s.Length; i++)
            {
                if (char.IsUpper(s, i))
                {
                    sb.Append('_');
                    sb.Append(char.ToLowerInvariant(s[i]));
                }
                else
                {
                    sb.Append(s[i]);
                }
            }
            return sb.ToString();
        }

        private bool CheckUnsupportedType(RsEndpoint endpoint)
        {
            using (var e = endpoint.RequiredParameters.Concat(endpoint.OptionalParameters)
                .Where(x => x.Type.Match(raw => raw.Type == "Stream", str => false, vec => false))
                .GetEnumerator())
            {
                if (e.MoveNext())
                {
                    writer.WriteLine("// Not Implemented: {0} ({1}: {2})", endpoint.Name, e.Current.Name, e.Current.Type);
                    return false;
                }
                else
                {
                    return true;
                }
            }
        }

        private void Header()
        {
            writer.Write(@"use std::rc::Rc;
use conn::Authenticator;

#[derive(Clone, Debug)]
pub struct TwitterClient<T: Authenticator>(pub Rc<T>);

impl<T: Authenticator> TwitterClient<T> {
    pub fn new(authenticator: T) -> TwitterClient<T> {
        TwitterClient(Rc::new(authenticator))
    }
");
        }

        private void ClientFactory(string clientName)
        {
            writer.Write(@"
    pub fn {0}(self) -> {1}Client<T> {{
        {1}Client(self.0)
    }}
",
                ToSnakeCase(clientName), clientName
            );
        }

        private void ClientStruct(string clientName)
        {
            writer.Write(@"
#[derive(Clone, Debug)]
pub struct {0}Client<T: Authenticator>(Rc<T>);
",
                clientName
            );
        }

        private static string GetSliceType(IRsType type)
        {
            return type.Match(raw => raw.Type, str => "&str", vec => string.Format("&[{0}]", vec.Type));
        }

        private static string SliceToVec(IRsType type)
        {
            return type.Match(raw => "", str => ".to_string()", vec => ".to_vec()");
        }

        private void ClientImpl(string clientName, IEnumerable<RsEndpoint> endpoints)
        {
            writer.WriteLine();
            writer.Write("impl<T: Authenticator> {0}Client<T> {{", clientName);

            foreach (var endpoint in endpoints)
            {
                if (!CheckUnsupportedType(endpoint)) continue;
                writer.WriteLine();
                writer.Write("    pub fn {0}(self", ToSnakeCase(endpoint.Name));
                foreach (var p in endpoint.RequiredParameters)
                    writer.Write(", {0}: {1}", p.Name, GetSliceType(p.Type));

                writer.Write(@") -> {0}{1}RequestBuilder<T> {{
        {0}{1}RequestBuilder {{
            _auth: self.0",
                    clientName, endpoint.Name
                );
                foreach (var p in endpoint.RequiredParameters)
                {
                    writer.WriteLine(',');
                    writer.Write("            {0}: {0}", p.Name);
                    writer.Write(SliceToVec(p.Type));
                }
                foreach (var p in endpoint.OptionalParameters)
                {
                    writer.WriteLine(',');
                    writer.Write("            {0}: None", p.Name);
                }

                writer.Write(@"
        }
    }
");
            }

            writer.WriteLine('}');
        }

        private void RequestBuilderStruct(string name, RsEndpoint endpoint)
        {
            writer.Write(@"
pub struct {0}<T: Authenticator> {{
    _auth: Rc<T>",
                name
            );
            foreach (var p in endpoint.RequiredParameters)
            {
                writer.WriteLine(',');
                writer.Write("    {0}: {1}", p.Name, p.Type);
            }
            foreach (var p in endpoint.OptionalParameters)
            {
                writer.WriteLine(',');
                writer.Write("    {0}: Option<{1}>", p.Name, p.Type);
            }
            writer.Write(@"
}
");
        }

        private void RequestBuilderImpl(string name, RsEndpoint endpoint)
        {
            writer.WriteLine();
            writer.Write("impl<T: Authenticator> {0}<T> {{", name);

            foreach (var p in endpoint.OptionalParameters)
            {
                writer.Write(@"
    pub fn {0}(mut self, val: {1}) -> Self {{
        self.{0} = Some(val{2});
        self
    }}
",
                    p.Name, GetSliceType(p.Type), SliceToVec(p.Type)
                );
            }

            Execute(endpoint);

            writer.WriteLine('}');
        }

        private void Execute(RsEndpoint endpoint)
        {
            //TODO
        }
    }
}
