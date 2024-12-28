let scene, camera, renderer, controls;

function init() {
  // 1. Create the scene
  scene = new THREE.Scene();

  // 2. Create a camera
  camera = new THREE.PerspectiveCamera(
    45, 
    window.innerWidth / window.innerHeight,
    0.1, 
    1000
  );
  // Position the camera so we can see the model
  camera.position.set(0, 2, 400);

  // 3. Create the renderer
  renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setSize(window.innerWidth, window.innerHeight);
  // Append the canvas to our container
  document.getElementById('canvas-container').appendChild(renderer.domElement);

  // 4. Add some lights
  const ambientLight = new THREE.AmbientLight(0xffffff, 0.5);
  scene.add(ambientLight);

  const directionalLight = new THREE.DirectionalLight(0xffffff, 0.5);
  directionalLight.position.set(5, 10, 7).normalize();
  scene.add(directionalLight);

  // 5. Add OrbitControls for mouse drag to rotate
  controls = new THREE.OrbitControls(camera, renderer.domElement);
  // Optional: set some limits, e.g. how close/far you can zoom:
  // controls.minDistance = 1;
  // controls.maxDistance = 20;

  // 6. Load the MTL and OBJ files
  const mtlLoader = new THREE.MTLLoader();
  // If your files are in a subfolder, set the path:
  // mtlLoader.setPath('./models/');

  mtlLoader.load(
    'models/model.mtl', 
    function (materials) {
      // Preload all materials
      materials.preload();

      // Now pass them to OBJLoader
      const objLoader = new THREE.OBJLoader();
      objLoader.setMaterials(materials);

      // If needed, set the path for the OBJ file as well:
      // objLoader.setPath('./models/');

      objLoader.load(
        'models/model.obj',
        function (obj) {
          // Optionally, center the model:
          // obj.position.set(0, 0, 0);

          // If the model is huge or too small, consider scaling:
          // obj.scale.set(0.5, 0.5, 0.5);

          scene.add(obj);
        },
        // onProgress callback
        function (xhr) {
          console.log((xhr.loaded / xhr.total) * 100 + '% loaded');
        },
        // onError callback
        function (error) {
          console.error('An error occurred while loading .obj:', error);
        }
      );
    },
    // onProgress callback
    function (xhr) {
      console.log('MTLLoader: ' + (xhr.loaded / xhr.total) * 100 + '% loaded');
    },
    // onError callback
    function (error) {
      console.error('An error occurred while loading .mtl:', error);
    }
  );

  // 7. Listen for window resizing
  window.addEventListener('resize', onWindowResize);

  // 8. Start animation loop
  animate();
}

function onWindowResize() {
  camera.aspect = window.innerWidth / window.innerHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(window.innerWidth, window.innerHeight);
}

function animate() {
  requestAnimationFrame(animate);

  // Let OrbitControls handle user interactions
  controls.update();

  // Render the scene
  renderer.render(scene, camera);
}

// Initialize the 3D scene
init();
