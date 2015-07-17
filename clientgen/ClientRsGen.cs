using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.RegularExpressions;
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


            //TODO: remove this
            foreach (var t in this.apis.SelectMany(a => a.Item2.Select(e => Tuple.Create(a.Item1, e))))
                if (t.Item2.Source.JsonPath != null)
                    writer.WriteLine("// Warning: {0}.{1} -> {2} has JSON path '{3}'", t.Item1, t.Item2.Name, t.Item2.Source.ReturnType, t.Item2.Source.JsonPath);
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
                    if (!CheckUnsupported(endpoint)) continue;
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

        private bool CheckUnsupported(RsEndpoint endpoint)
        {
            if (endpoint.ReturnType == null)
            {
                writer.WriteLine("// Warning: Unsupported Return Type: {0} -> {1}", endpoint.Name, endpoint.Source.ReturnType);
                return false;
            }

            using (var e = endpoint.RequiredParameters.Concat(endpoint.OptionalParameters)
                .Where(x => x.Type.Match(raw => raw.Type == "Stream", str => false, vec => false, unit => true))
                .GetEnumerator())
            {
                if (e.MoveNext())
                {
                    writer.WriteLine("// Warning: Not Implemented: {0} ({1}: {2})", endpoint.Name, e.Current.Name, e.Current.Type);
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
use hyper::{Get, Post};
use ::{TwitterError, TwitterResult};
use conn::{Authenticator, Parameter, parse_json};
use models::CursorIds;
use models::direct_messages::DirectMessage;
use models::friendships::{Friendship, Relationship};
use models::places::Place;
use models::search::SearchResponse;
use models::tweets::{OEmbed, Tweet};
use models::users::{CursorUsers, User};

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

        private static string GetSliceType(RsType type)
        {
            return type.Match(raw => raw.Type, str => "&str", vec => string.Format("&[{0}]", vec.Type), unit => unit.ToString());
        }

        private static string SliceToVec(RsType type)
        {
            return type.Match(raw => "", str => ".to_string()", vec => ".to_vec()", unit => "");
        }

        private void ClientImpl(string clientName, IEnumerable<RsEndpoint> endpoints)
        {
            writer.WriteLine();
            writer.Write("impl<T: Authenticator> {0}Client<T> {{", clientName);

            foreach (var endpoint in endpoints)
            {
                if (!CheckUnsupported(endpoint)) continue;
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

        private static string ParameterFactory(RsType t)
        {
            if (t is VecType) return "from_vec";
            return "key_value";
        }

        private void Execute(RsEndpoint endpoint)
        {
            writer.WriteLine();
            writer.WriteLine("    pub fn execute(&self) -> TwitterResult<{0}> {{", endpoint.ReturnType);

            var needsFormat = endpoint.ReservedParameter != null;
            var capacity = endpoint.RequiredParameters.Count + endpoint.OptionalParameters.Count;
            if (needsFormat) capacity--;
            if (capacity == 0) writer.WriteLine("        let params = Vec::<Parameter>::new();");
            else writer.WriteLine("        let mut params = Vec::with_capacity({0});", capacity);

            foreach (var p in endpoint.RequiredParameters)
            {
                if (p.Name != endpoint.ReservedParameter)
                    writer.WriteLine(
                        @"        params.push(Parameter::{0}(""{1}"", self.{1}.clone()));",
                        ParameterFactory(p.Type), p.Name);
            }

            foreach (var p in endpoint.OptionalParameters)
            {
                writer.Write(@"        match self.{0} {{
            Some(ref x) => params.push(Parameter::{1}(""{0}"", x.clone())),
            None => ()
        }}
",
                    p.Name, ParameterFactory(p.Type)
                );
            }

            writer.Write("        let url = ");
            if (needsFormat)
                writer.WriteLine(
                    @"format!(""https://api.twitter.com/1.1/{0}.json"", self.{1});",
                    Regex.Replace(endpoint.Uri, @"\{\w+\}", "{}"),
                    endpoint.ReservedParameter
                );
            else
                writer.WriteLine(@"""https://api.twitter.com/1.1/{0}.json"";", endpoint.Uri);

            writer.Write(@"        let res = try!(Authenticator::request_twitter(&*self._auth, {0}, {1}, &params[..]));
        match parse_json(&res.raw_response[..]) {{
            Ok(j) => Ok(res.object(j)),
            Err(e) => Err(TwitterError::JsonError(e, res))
        }}
    }}
",
                endpoint.Method, needsFormat ? "&url[..]" : "url"
            );
        }
    }
}
