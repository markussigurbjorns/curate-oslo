let scene, camera, renderer, myModel;

function init() {
  scene = new THREE.Scene();

  camera = new THREE.PerspectiveCamera(
    45, 
    window.innerWidth / window.innerHeight, 
    0.1, 
    1000
  );
  camera.position.z = 5;

  renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setSize(window.innerWidth, window.innerHeight);

  document.getElementById('canvas-container').appendChild(renderer.domElement);

  // Add some light
  const ambientLight = new THREE.AmbientLight(0xffffff, 0.4);
  scene.add(ambientLight);

  const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
  directionalLight.position.set(1, 2, 3).normalize();
  scene.add(directionalLight);

  // ---- Load the MTL and OBJ files ----
  const mtlLoader = new THREE.MTLLoader();
  // optional: set a path if model files are in a subfolder
  // mtlLoader.setPath('./path/to/folder/'); 

  mtlLoader.load('model.mtl', function (materials) {
    // Preload the materials
    materials.preload();

    // Now set the materials in OBJLoader
    const objLoader = new THREE.OBJLoader();
    objLoader.setMaterials(materials);

    // optional: set the same path
    // objLoader.setPath('./path/to/folder/'); 

    objLoader.load(
      'model.obj',
      function (object) {
        myModel = object;
        myModel.scale.set(0.3, 0.3, 0.3);
        myModel.position.set(0, -1, 0);
        myModel.rotation.x = Math.PI / 3/4; 
        scene.add(myModel);
      },
      function (xhr) {
        console.log((xhr.loaded / xhr.total * 100) + '% loaded');
      },
      function (error) {
        console.error('An error happened while loading OBJ:', error);
      }
    );
  });

  window.addEventListener('resize', onWindowResize);

  animate();
}

function onWindowResize() {
  camera.aspect = window.innerWidth / window.innerHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(window.innerWidth, window.innerHeight);
}

function animate() {
  requestAnimationFrame(animate);

  // If the model is loaded, rotate it
  if (myModel) {
    // Rotate around X, Y, and Z axes
    //myModel.rotation.x += 0.01;  // rotate around X
    myModel.rotation.y += 0.01;  // rotate around Y
    // myModel.rotation.z += 0.01; // rotate around Z if you like
  }

  renderer.render(scene, camera);
}

init();
