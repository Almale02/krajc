struct Vertex {
  float3 pos;
  float3 color;
};
struct PushConstants {
  Vertex *vertexBufferAddress;
};
struct VertexOutput {
  float4 svPos : SV_Position;
  float3 color;
};

[shader("vertex")]
VertexOutput vertexMain(uint vertexId: SV_VertexID,
                        uniform PushConstants constants) {
  VertexOutput output;
  Vertex v = constants.vertexBufferAddress[vertexId];
  output.svPos = float4(v.pos, 1.0);
  output.color = v.color;
  return output;
};
[shader("fragment")]
float4 fragmentMain(VertexOutput input) : SV_Target {
  return float4(input.color, 1.0);
}
