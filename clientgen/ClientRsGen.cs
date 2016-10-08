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

            if (endpoint.Method == "Impl")
            {
                writer.WriteLine("// Warning: {0} requires Impl method", endpoint.Name);
                return false;
            }

            using (var e = endpoint.RequiredParameters.Concat(endpoint.OptionalParameters)
                .Where(x => x.Type.Match(raw => raw.Type == "Stream", str => false, vec => false, unit => true))
                .GetEnumerator())
            {
                if (e.MoveNext())
                {
                    writer.WriteLine("// Warning: Mutltipart is Not Implemented: {0} ({1}: {2})", endpoint.Name, e.Current.Name, e.Current.Type);
                    return false;
                }
            }

            return true;
        }

        private void Header()
        {
            writer.Write(@"use std::borrow::Cow;
use hyper::{Get, Post};
use ::TwitterResult;
use conn::{Authenticator, Parameter};
use models::CursorIds;
use models::direct_messages::DirectMessage;
use models::friendships::{Friendship, Relationship};
use models::places::Place;
use models::search::SearchResponse;
use models::tweets::{OEmbed, Tweet};
use models::users::{CursorUsers, User};
use self::helper::*;
use self::request::*;

mod helper;
pub mod request;

#[derive(Clone, Debug)]
pub struct TwitterClient<T: Authenticator> { auth: T }

impl<T: Authenticator> TwitterClient<T> {
    pub fn new(authenticator: T) -> TwitterClient<T> {
        TwitterClient { auth: authenticator }
    }
");
        }

        private void ClientFactory(string clientName)
        {
            writer.Write(@"
    pub fn {0}(&self) -> {1}Client<T> {{
        {1}Client {{ auth: &self.auth }}
    }}
",
                ToSnakeCase(clientName), clientName
            );
        }

        private void ClientStruct(string clientName)
        {
            writer.Write(@"
#[derive(Clone, Debug)]
pub struct {0}Client<'a, T: Authenticator + 'a> {{ auth: &'a T }}
",
                clientName
            );
        }

        private static string FieldType(RsType type) => type.Match(
            raw => raw.Type,
            str => "Cow<'a, str>",
            vec => "String",
            unitType => { throw new ArgumentException(); });

        private void ClientImpl(string clientName, IEnumerable<RsEndpoint> endpoints)
        {
            writer.WriteLine();
            writer.Write("impl<'a, T: Authenticator> {0}Client<'a, T> {{", clientName);

            foreach (var endpoint in endpoints)
            {
                GetBuilderFunc(clientName, endpoint);
            }

            writer.WriteLine('}');
        }

        private void GetBuilderFunc(string clientName, RsEndpoint endpoint)
        {
            if (!CheckUnsupported(endpoint)) return;

            var typeParameters = new List<string>();
            var sb = new StringBuilder();
            foreach (var p in endpoint.RequiredParameters)
            {
                sb.AppendFormat(", {0}: {1}", p.Name, p.Type.Match(
                    raw => raw.Type,
                    str =>
                    {
                        typeParameters.Add("Into<Cow<'a, str>>");
                        return "T" + typeParameters.Count;
                    },
                    vec =>
                    {
                        if (vec.Type is StringType)
                        {
                            // IntoIterator<Item=AsRef<str>>
                            typeParameters.Add("AsRef<str>");
                            typeParameters.Add($"IntoIterator<Item=T{typeParameters.Count}>");
                        }
                        else
                        {
                            typeParameters.Add($"IntoIterator<Item={vec.Type}>");
                        }

                        return "T" + typeParameters.Count;
                    },
                    unit => { throw new InvalidOperationException(); }
                ));
            }

            writer.WriteLine();
            writer.Write("    pub fn {0}", ToSnakeCase(endpoint.Name));

            if (typeParameters.Count > 0)
            {
                writer.Write('<');
                writer.Write(string.Join(", ", typeParameters.Select((x, i) => $"T{i + 1}: {x}")));
                writer.Write('>');
            }

            writer.Write("(&self");
            writer.Write(sb);

            writer.Write(@") -> {0}{1}RequestBuilder<'a, T> {{
        {0}{1}RequestBuilder {{
            _auth: self.auth",
                clientName, endpoint.Name
            );
            foreach (var p in endpoint.RequiredParameters)
            {
                writer.WriteLine(',');
                writer.Write("            {0}: ", p.Name);
                writer.Write(
                    p.Type.Match(
                        raw => "{0}",
                        str => "{0}.into()",
                        vec => vec.Type is StringType ? "str_collection_parameter({0})" : "collection_paramter({0})",
                        unit => { throw new InvalidOperationException(); }
                    ),
                    p.Name
                );
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

        private void RequestBuilderStruct(string name, RsEndpoint endpoint)
        {
            writer.Write(@"
pub struct {0}<'a, T: Authenticator + 'a> {{
    _auth: &'a T",
                name
            );
            foreach (var p in endpoint.RequiredParameters)
            {
                writer.WriteLine(',');
                writer.Write("    {0}: {1}", p.Name, FieldType(p.Type));
            }
            foreach (var p in endpoint.OptionalParameters)
            {
                writer.WriteLine(',');
                writer.Write("    {0}: Option<{1}>", p.Name, FieldType(p.Type));
            }
            writer.Write(@"
}
");
        }

        private void RequestBuilderImpl(string name, RsEndpoint endpoint)
        {
            writer.WriteLine();
            writer.Write("impl<'a, T: Authenticator> {0}<'a, T> {{", name);

            foreach (var p in endpoint.OptionalParameters)
            {
                var typeParameters = new List<string>();
                var parameterType = p.Type.Match(
                    raw => raw.Type,
                    str =>
                    {
                        typeParameters.Add("Into<Cow<'a, str>>");
                        return "T" + typeParameters.Count;
                    },
                    vec =>
                    {
                        if (vec.Type is StringType)
                        {
                            // IntoIterator<Item=AsRef<str>>
                            typeParameters.Add("AsRef<str>");
                            typeParameters.Add($"IntoIterator<Item=T{typeParameters.Count}>");
                        }
                        else
                        {
                            typeParameters.Add($"IntoIterator<Item={vec.Type}>");
                        }

                        return "T" + typeParameters.Count;
                    },
                    unit => { throw new InvalidOperationException(); }
                );

                writer.WriteLine();
                writer.Write("    pub fn {0}", p.Name);

                if (typeParameters.Count > 0)
                {
                    writer.Write('<');
                    writer.Write(string.Join(", ", typeParameters.Select((x, i) => $"T{i + 1}: {x}")));
                    writer.Write('>');
                }

                writer.Write(@"(&mut self, val: {0}) -> &mut Self {{
        self.{1} = Some({2});
        self
    }}
",
                    parameterType,
                    p.Name,
                    p.Type.Match(
                        raw => "val",
                        str => "val.into()",
                        vec => vec.Type is StringType ? "str_collection_parameter(val)" : "collection_paramter(val)",
                        unit => { throw new InvalidOperationException(); }
                    )
                );
            }

            Execute(endpoint);

            writer.WriteLine('}');
        }

        private static string ConvertParameterFuncName(RsType type) =>
            type.Match(
                raw => raw.Type == "bool" ? "bool_parameter"
                    : raw.Type == "TweetMode" ? "tweet_mode_parameter"
                    : "parameter",
                str => "cow_str_parameter",
                vec => "owned_str_parameter",
                unit => { throw new ArgumentException(); }
            );

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
                        @"        params.push({1}(""{0}"", &self.{0}));",
                        p.Name, ConvertParameterFuncName(p.Type));
            }

            foreach (var p in endpoint.OptionalParameters)
            {
                writer.WriteLine(
                    @"        if let Some(ref x) = self.{0} {{ params.push({1}(""{0}"", x)) }}",
                    p.Name, ConvertParameterFuncName(p.Type)
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

            writer.Write(@"        execute_core(self._auth, {0}, url, &params)
    }}
",
                endpoint.Method
            );
        }
    }
}
